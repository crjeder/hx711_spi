#![doc = include_str!("../README.md")]

#![forbid(unsafe_code)]
#![no_std]

use bitmatch::bitmatch;
use core::unimplemented;
use embedded_hal as hal;
use hal::blocking::delay::DelayMs;
use hal::blocking::spi;
use nb::{self, block};

/// The HX711 has two channels: `A` for the load cell and `B` for AD conversion of other signals.
/// Channel `A` supports gains of 128 (default) and 64, `B` has a fixed gain of 32.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Mode {
    // bits have to be converted for correct transfer 1 -> 10, 0 -> 00
    /// Convert channel A with a gain factor of 128
    ChAGain128 = 0b1000_0000,
    /// Convert channel B with a gain factor of 32
    ChBGain32 = 0b1010_0000,
    /// Convert channel A with a gain factor of 64
    ChAGain64 = 0b1010_1000, // there is a typo in the official datasheet: in Fig.2 it says channel B instead of A
}

/// Represents an instance of a HX711 device
#[derive(Debug)]
pub struct Hx711<SPI, D> {
    // SPI specific
    spi: SPI,
    // device specific
    mode: Mode,
    // timer for delay
    delay: D,
}

impl<SPI, E, D> Hx711<SPI, D>
where
    SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
    D: DelayMs<u16>,
{
    /// opens a connection to a HX711 on a specified SPI.
    ///
    /// The datasheet specifies PD_SCK high time and PD_SCK low time to be in the 0.2 to 50 us range,
    /// therefore bus speed has to be between 5 MHz and 20 kHz. 1 MHz seems to be a good choice.
    /// D is an embedded_hal implementation of DelayMs.
    pub fn new(spi: SPI, delay: D) -> Self {
        Hx711 {
            spi,
            mode: Mode::ChAGain128,
            delay,
        }
    }

    /// reads a value from the HX711 and returns it
    /// # Errors
    /// Returns SPI errors and nb::Error::WouldBlock if data isn't ready to be read from hx711
    pub fn read(&mut self) -> nb::Result<i32, E> {
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high.
        // Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let mut txrx: [u8; 1] = [0];

        self.spi.transfer(&mut txrx)?;

        if txrx[0] & 0b01 == 0b01 {
            // as long as the lowest bit is high there is no data waiting 
            return Err(nb::Error::WouldBlock);
        }

        // the read has the same length as the write.
        // SDO provides clock to the HX711's shift register (binary 1010...)
        // one clock cycle is '10'. The buffer needs to be double the size of the 4 bytes we want to read
        const CLOCK: u8 = 0b10101010;

        let mut buffer: [u8; 7] = [
            CLOCK,
            CLOCK,
            CLOCK,
            CLOCK,
            CLOCK,
            CLOCK,
            self.mode as u8,
        ];

        self.spi.transfer(&mut buffer)?;
        // value should be in range 0x800000 - 0x7fffff according to datasheet

        Ok(decode_output(&buffer))
    }

#[inline]
    /// This is for compatibility only. Use [read]() instead.
    pub fn retrieve(&mut self) -> nb::Result<i32, E> {
        self.read()
    }
    /// Reset the chip to it's default state. Mode is set to convert channel A with a gain factor of 128.
    /// # Errors
    /// Returns SPI errors
#[inline]
    pub fn reset(&mut self) -> Result<(), E> {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs,
        // HX711 enters power down mode.
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // speed is the raw SPI speed -> half bits per second.

        // max SPI clock frequency should be 5 MHz to satisfy the 0.2 us limit for the pulse length
        // we have to output more than 300 bytes to keep the line for at least 60 us high.

        let buffer: [u8; 301] = [0xFF; 301];

        self.spi.write(&buffer)?;
        self.mode = Mode::ChAGain128; // this is the default mode after reset

        Ok(())
    }

    /// Set the mode to the value specified.
    /// # Errors
    /// Returns SPI errors
#[inline]
    pub fn set_mode(&mut self, m: Mode) -> Result<Mode, E> {
        self.mode = m;
        block!(self.read())?; // read writes Mode for the next read()
        Ok(m)
    }

#[inline]
    /// Get the current mode.
    pub fn mode(&mut self) -> Mode {
        self.mode
    }
    
#[inline]
    /// This is for compatibility only. Use [mode]() instead.
    pub fn get_mode(&mut self) -> Mode {
        self.mode
    }

    /// To power down the chip the PD_SCK line has to be held in a 'high' state. To do this we
    /// would need to write a constant stream of binary '1' to the SPI bus which would totally defy
    /// the purpose. Therefore it's not implemented.
    // If the SDO pin would be idle high (and at least some MCU's seem to do that in mode 1) then the chip would automatically
    // power down if not used. Cool!
    pub fn disable(&mut self) -> Result<(), E> {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // this can't be implemented with SPI because we would have to write a constant stream
        // of binary '1' which would block the process
        unimplemented!("power_down is not possible with this driver implementation");
    }

    /// Power up / down is not implemented (see disable)
    pub fn enable(&mut self) -> Result<(), E> {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // this can't be implemented with SPI because we would have to write a constant stream
        // of binary '1' which would block the process
        unimplemented!("power_down is not possible with this driver implementation");
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
