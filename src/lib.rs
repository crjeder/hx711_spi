#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![no_std]

use bitmatch::bitmatch;
use embedded_hal as hal;
use embedded_hal_async::spi::SpiBus as AsyncSpiBus;
use hal::spi::SpiBus;
/// The absolute minimum reading. A lesser value should be clamped.
pub const HX711_MINIMUM: i32 = -(2i32.saturating_pow(24 - 1));
/// The absolute maximum reading. A greater value should be clamped.
pub const HX711_MAXIMUM: i32 = 2i32.saturating_pow(24 - 1) - 1;

// Bit pattern definitions for the communication with the hx711. All have to be bitwise negated
// for the ```invert-sdo``` feature

// patterns for mode
#[cfg(not(feature = "invert-sdo"))]
const GAIN128: u8 = 0b1000_0000;
#[cfg(feature = "invert-sdo")]
const GAIN128: u8 = 0b0111_1111;

#[cfg(not(feature = "invert-sdo"))]
const GAIN32: u8 = 0b1010_0000;
#[cfg(feature = "invert-sdo")]
const GAIN32: u8 = 0b0101_1111;

#[cfg(not(feature = "invert-sdo"))]
const GAIN64: u8 = 0b1010_1000;
#[cfg(feature = "invert-sdo")]
const GAIN64: u8 = 0b0101_0111;

// SDO provides clock to the HX711's shift register (binary 1010...)
// one clock cycle is '10'. The buffer needs to be double the size of the 4 bytes we want to read
#[cfg(not(feature = "invert-sdo"))]
const CLOCK: u8 = 0b10101010;
#[cfg(feature = "invert-sdo")]
const CLOCK: u8 = 0b01010101;

// Signal to send to the HX711 when checking for data ready to be read.
#[cfg(not(feature = "invert-sdo"))]
const SIGNAL_LOW: u8 = 0x0;
#[cfg(feature = "invert-sdo")]
const SIGNAL_LOW: u8 = 0xFF;

// reset signal
#[cfg(not(feature = "invert-sdo"))]
const RESET_SIGNAL: [u8; 301] = [0xFF; 301];
#[cfg(feature = "invert-sdo")]
const RESET_SIGNAL: [u8; 301] = [0x00; 301];

// End bit pattern definitions

/// The HX711 has two channels: `A` for the load cell and `B` for AD conversion of other signals.
/// Channel `A` supports gains of 128 (default) and 64, `B` has a fixed gain of 32.
/// Set chanel and gain with the set_mode function.
///
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Mode {
    // bits have to be converted for correct transfer 1 -> 10, 0 -> 00
    /// Convert channel A with a gain factor of 128
    ChAGain128 = GAIN128,
    /// Convert channel A with a gain factor of 64
    ChAGain64 = GAIN64, // there is a typo in the official datasheet: in Fig.2 it says channel B instead of A
    /// Convert channel B with a gain factor of 32
    ChBGain32 = GAIN32,
}

/// Represents an instance of a HX711 device
#[derive(Debug)]
pub struct Hx711<SPI> {
    // SPI specific
    spi: SPI,
    // device specific
    mode: Mode,
}

#[derive(Copy, Clone, Debug)]
pub enum Hx711Error<SPI> {
    Spi(SPI),
    DataNotReady,
}

impl<SPIERROR> From<SPIERROR> for Hx711Error<SPIERROR> {
    fn from(value: SPIERROR) -> Self {
        Hx711Error::Spi(value)
    }
}

impl<SPI> Hx711<SPI> {
    /// Get the current mode.
    #[inline]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Construct a new `Hx711` with the default mode (`ChAGain128`).
    pub fn new(spi: SPI) -> Self {
        Hx711 {
            spi,
            mode: Mode::ChAGain128,
        }
    }

    fn mode_buffer(&self) -> [u8; 7] {
        [CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, self.mode as u8]
    }

    /// To power down the chip the PD_SCK line has to be held in a 'high' state. To do this we
    /// would need to write a constant stream of binary '1' to the `SPI` bus which would totally defy
    /// the purpose. Therefore it's not implemented.
    // If the SDO pin would be idle high (and at least some MCU's seem to do that in mode 1) then the chip would automatically
    // power down if not used. Cool!
    pub fn disable(&mut self) -> ! {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // this can't be implemented with SPI because we would have to write a constant stream
        // of binary '1' which would block the process
        unimplemented!("power_down is not possible with this driver implementation");
    }

    /// Power up is not implemented (see `disable`).
    pub fn enable(&mut self) -> ! {
        unimplemented!("power_up is not possible with this driver implementation");
    }
}

impl<SPI> Hx711<SPI>
where
    SPI: SpiBus,
{
    /// reads a value from the HX711 and returns it
    /// # Errors
    /// Returns `Hx711Error::DataNotReady` if data isn't ready, or `Hx711Error::Spi` on bus errors.
    pub fn read(&mut self) -> Result<i32, Hx711Error<SPI::Error>> {
        let mut txrx: [u8; 1] = [SIGNAL_LOW];
        self.spi.transfer_in_place(&mut txrx)?;

        if txrx[0] == SIGNAL_LOW {
            let mut buffer = self.mode_buffer();
            self.spi.transfer_in_place(&mut buffer)?;
            Ok(decode_output(&buffer))
        } else {
            Err(Hx711Error::DataNotReady)
        }
    }

    /// Reset the chip to its default state. Mode is set to `ChAGain128`.
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub fn reset(&mut self) -> Result<(), Hx711Error<SPI::Error>> {
        let mut buffer: [u8; 301] = RESET_SIGNAL;
        self.spi.transfer_in_place(&mut buffer)?;
        self.mode = Mode::ChAGain128;
        Ok(())
    }

    /// Set the mode to the value specified.
    /// see the Mode struct for possible values
    /// # Usage
    ///
    /// ```rust,ignore
    /// my_hx711.set_mode(Mode::ChAGain128);
    /// value1_chanel_a = my_hx711.read()?
    /// value2_chanel_a = my_hx711.read()?
    /// my_hx711.set_mode(Mode::ChBGain32);
    /// value_chanel_b = my_hx711.read()?
    ///```
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub fn set_mode(&mut self, m: Mode) -> Result<Mode, Hx711Error<SPI::Error>> {
        self.mode = m;
        self.read()?; // read writes Mode for the next read()
        Ok(m)
    }
}

impl<SPI> Hx711<SPI>
where
    SPI: AsyncSpiBus,
{
    /// Polls until data is ready, then reads and returns the value.
    /// # Errors
    /// Returns `SPI` errors
    pub async fn read_async(&mut self) -> Result<i32, SPI::Error> {
        let mut txrx = [SIGNAL_LOW];
        loop {
            self.spi.transfer_in_place(&mut txrx).await?;
            if txrx[0] == SIGNAL_LOW { break; }
            txrx[0] = SIGNAL_LOW; // restore probe value for next iteration
        }

        let mut buffer = self.mode_buffer();
        self.spi.transfer_in_place(&mut buffer).await?;
        Ok(decode_output(&buffer))
    }

    /// Reset the chip to its default state. Mode is set to `ChAGain128`.
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub async fn reset_async(&mut self) -> Result<(), SPI::Error> {
        let mut buffer: [u8; 301] = RESET_SIGNAL;
        self.spi.transfer_in_place(&mut buffer).await?;
        self.mode = Mode::ChAGain128;
        Ok(())
    }

    /// Set the mode to the value specified.
    /// see the Mode struct for possible values
    /// # Usage
    ///
    /// ```rust,ignore
    /// my_hx711.set_mode_async(Mode::ChAGain128).await?;
    /// value1_chanel_a = my_hx711.read_async().await?;
    /// value2_chanel_a = my_hx711.read_async().await?;
    /// my_hx711.set_mode_async(Mode::ChBGain32).await?;
    /// value_chanel_b = my_hx711.read_async().await?;
    ///```
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub async fn set_mode_async(&mut self, m: Mode) -> Result<Mode, SPI::Error> {
        self.mode = m;
        self.read_async().await?; // read writes Mode for the next read()
        Ok(m)
    }
}

#[bitmatch]
fn decode_output(buffer: &[u8; 7]) -> i32 {
    // buffer contains the 2's complement of the reading with every bit doubled
    // since the first byte is the most significant it's big endian
    // we have to extract every second bit from the buffer
    // only the upper 24 (doubled) bits are valid

    #[bitmatch]
    let "a?a?a?a?" = buffer[0];
    #[bitmatch]
    let "b?b?b?b?" = buffer[1];
    #[bitmatch]
    let "c?c?c?c?" = buffer[2];
    #[bitmatch]
    let "d?d?d?d?" = buffer[3];
    #[bitmatch]
    let "e?e?e?e?" = buffer[4];
    #[bitmatch]
    let "f?f?f?f?" = buffer[5];

    let raw: [u8; 4] = [
        bitpack!("aaaabbbb"),
        bitpack!("ccccdddd"),
        bitpack!("eeeeffff"),
        0,
    ];

    i32::from_be_bytes(raw) / 0x100
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55] => 0; "alternating convert to zeros")]
    #[test_case(&[0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA] => -1; "alternating convert to ones")]
    #[test_case(&[0xFF, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF] => -1; "all ones")]
    #[test_case(&[0b00100111, 0b00100111, 0b00100111, 0b00100111,
                  0b00100111, 0b00100111, 0b00100111] => 0b0000_0000_0101_0101_0101_0101_0101_0101i32; "test pattern")]
    fn test_decode(buffer: &[u8; 7]) -> i32 {
        decode_output(buffer)
    }
}
