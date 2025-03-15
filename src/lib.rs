#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![no_std]

use bitmatch::bitmatch;
use core::unimplemented;
use embedded_hal as hal;
use embedded_hal_async::spi::SpiBus as AsyncSpiBus;
use hal::spi::SpiBus;
//
// saturation
// values above maximum and below minimum are wrong
pub const HX711_MINIMUM: i32 = -(2i32.saturating_pow(24 - 1));
/// The absolute maximum readings. A greater value should be clamped.
pub const HX711_MAXIMUM: i32 = 2i32.saturating_pow(24 - 1) - 1;
// if signed < HX711_MINIMUM {
//    signed = HX711_MINIMUM;
//} else if signed > HX711_MAXIMUM {

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
    #[inline]
    /// Get the current mode.
    pub fn mode(&mut self) -> Mode {
        self.mode
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

    /// Power up / down is not implemented (see disable)
    pub fn enable(&mut self) -> ! {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // this can't be implemented with SPI because we would have to write a constant stream
        // of binary '1' which would block the process
        unimplemented!("power_down is not possible with this driver implementation");
    }
}

impl<SPI> Hx711<SPI>
where
    SPI: SpiBus,
{
    /// opens a connection to a HX711 on a specified `SPI`.
    ///
    /// The data sheet specifies PD_SCK high time and PD_SCK low time to be in the 0.2 to 50 us range,
    /// therefore bus speed has to be between 5 MHz and 20 kHz.
    pub fn new(spi: SPI) -> Self {
        Hx711 {
            spi,
            mode: Mode::ChAGain128,
        }
    }

    /// reads a value from the HX711 and returns it
    /// # Errors
    /// Returns `SPI` errors and `nb`::Error::`WouldBlock` if data isn't ready to be read from hx711
    pub fn read(&mut self) -> Result<i32, Hx711Error<SPI::Error>> {
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high.
        // Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let mut txrx: [u8; 1] = [SIGNAL_LOW];

        self.spi.transfer_in_place(&mut txrx)?;

        if txrx[0] == 0x00 {
            let mut buffer: [u8; 7] = [CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, self.mode as u8];

            self.spi.transfer_in_place(&mut buffer)?;

            Ok(decode_output(&buffer)) // value should be in range 0x800000 - 0x7fffff according to datasheet
        } else {
            Err(Hx711Error::DataNotReady)
        }
    }

    /// Reset the chip to it's default state. Mode is set to convert channel A with a gain factor of 128.
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub fn reset(&mut self) -> Result<(), Hx711Error<SPI::Error>> {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs,
        // HX711 enters power down mode.
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // speed is the raw SPI speed -> half bits per second.

        // max SPI clock frequency should be 5 MHz to satisfy the 0.2 us limit for the pulse length
        // we have to output more than 300 bytes to keep the line for at least 60 us high.

        let mut buffer: [u8; 301] = RESET_SIGNAL;

        self.spi.transfer_in_place(&mut buffer)?;
        self.mode = Mode::ChAGain128; // this is the default mode after reset

        Ok(())
    }

    /// Set the mode to the value specified.
    /// see the Mode struct for possible values
    /// # Usage
    ///
    /// ```text
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
    /// opens a connection to a HX711 on a specified `SPI`.
    ///
    /// The data sheet specifies PD_SCK high time and PD_SCK low time to be in the 0.2 to 50 us range,
    /// therefore bus speed has to be between 5 MHz and 20 kHz.
    pub fn new_async(spi: SPI) -> Self {
        Hx711 {
            spi,
            mode: Mode::ChAGain128,
        }
    }

    /// reads a value from the HX711 and returns it
    /// # Errors
    /// Returns `SPI` errors
    pub async fn read_async(&mut self) -> Result<i32, SPI::Error> {
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high.
        // Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let mut txrx: [u8; 1] = [SIGNAL_LOW];

        while txrx[0] != 0x00 {
            self.spi.transfer_in_place(&mut txrx).await?;
        }

        let mut buffer: [u8; 7] = [CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, CLOCK, self.mode as u8];

        self.spi.transfer_in_place(&mut buffer).await?;

        Ok(decode_output(&buffer)) // value should be in range 0x800000 - 0x7fffff according to datasheet
    }

    /// Reset the chip to it's default state. Mode is set to convert channel A with a gain factor of 128.
    /// # Errors
    /// Returns `SPI` errors
    #[inline]
    pub async fn reset_async(&mut self) -> Result<(), SPI::Error> {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs,
        // HX711 enters power down mode.
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // speed is the raw SPI speed -> half bits per second.

        // max SPI clock frequency should be 5 MHz to satisfy the 0.2 us limit for the pulse length
        // we have to output more than 300 bytes to keep the line for at least 60 us high.

        let mut buffer: [u8; 301] = RESET_SIGNAL;

        self.spi.transfer_in_place(&mut buffer).await?;
        self.mode = Mode::ChAGain128; // this is the default mode after reset

        Ok(())
    }

    /// Set the mode to the value specified.
    /// see the Mode struct for possible values
    /// # Usage
    ///
    /// ```rust
    /// my_hx711.set_mode_async(Mode::ChAGain128).await?;
    /// value1_chanel_a = my_hx711.read_async().await?
    /// value2_chanel_a = my_hx711.read_async().await?
    /// my_hx711.set_mode_async(Mode::ChBGain32).await?;
    /// value_chanel_b = my_hx711.read_async().await?
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

    let mut raw: [u8; 4] = [0; 4];
    raw[0] = bitpack!("aaaabbbb");
    raw[1] = bitpack!("ccccdddd");
    raw[2] = bitpack!("eeeeffff");
    raw[3] = 0;

    i32::from_be_bytes(raw) / 0x100
}

#[cfg(test)]
//#[macro_use]
//extern crate std;
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55] => 0; "alternating convert to zeros")]
    #[test_case(&[0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA] => -1; "alternating convert to ones")]
    #[test_case(&[0xFF, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF] => -1; "all ones")]
    #[test_case(&[0b00100111, 0b00100111, 0b00100111, 0b00100111,
                  0b00100111, 0b00100111, 0b00100111] => 0b0000_0000_0101_0101_0101_0101_0101_0101i32; "test pattern")]
    fn test_decode(buffer: &[u8; 7]) -> i32 {
        decode_output(&buffer)
    }
}
