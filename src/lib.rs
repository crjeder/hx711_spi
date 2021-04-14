//! HX711 embedded-hal SPI driver crate
//!
//! This is a platform agnostic driver to interface with the HX711 load cell IC. It uses SPI instad of bit banging.
//! This driver is built using [`embedded-hal`][2] traits.
//!
//!
//! # Usage
//! Use an embedded-hal implementation to get SPI. HX711 does not use CS and SCLK. Make sure that it
//! is the only device on the bus. Connect the SDO to the PD_SCK and SDI to DOUT of the HX711. SPI
//!  clock frequency has to be between 20 kHz and 5 MHz.
//!
//! ```rust
//! use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
//! use hx711_spi::{Hx711, HX711Mode};
//!
//! let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
//!
//! // to create sensor with default configuration:
//! let mut scale = Hx711(spi);
//!
//! // start measurements
//! let mut value = scale.readout().unwrap();
//! ```
//!
//! # References
//!
//! - [Datasheet][1]
//!
//! [1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf
//!
//! - [`embedded-hal`][2]
//!
//! [2]: https://github.com/rust-embedded/embedded-hal
//!
//!


// #![no_std]

use std::time::Duration;
use std::thread::sleep;

use embedded_hal as hal;
use hal::blocking::spi;

// use bitmach to decode the result
use bitmatch::bitmatch;

/// The HX711 has two chanels: `A` for the load cell and `B` for AD conversion of other signals.
/// Channel `A` supports gains of 128 (default) and 64, `B` has a fixed gain of 32.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum HX711Mode {
    // bits have to be converted for correct transfer 1 -> 10, 0 -> 00
    /// Convet channel A with a gain factor of 128
    ChAGain128 = 0b1000000,
    /// Convert channel B with a gain factor of 64
    ChBGain32 = 0b10100000,
    /// Convert channel A with a gain factor of 32
    ChAGain64 = 0b10101000
    // there is a typo in the official datasheet: in Fig.2 it says channel B instead of A
}

/// Represents an instance of a HX711 device
#[derive(Debug)]
pub struct Hx711<SPI>
{
    // SPI specific
    spi: SPI,
    // device specific
    mode: HX711Mode
}

impl <SPI, E> Hx711<SPI>
where
    SPI: spi::Transfer<u8, Error=E> + spi::Write<u8, Error=E>
{
    /// opens a connection to a HX711 on a specified SPI
    pub fn new(spi:SPI) -> Result<Self, E>
    {
        // datasheet specifies PD_SCK high time and PD_SCK low time to be in the 0.2 to 50 us range
        // therefore bus speed is 5 MHz to 20 kHz. 1 MHz seems to be a good choice
        // let dev = Spi::new(bus, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;

        Ok
        (
            Hx711
            {
                spi,
                mode: HX711Mode::ChAGain128,
            }
        )
    }

    /// reads a value from the HX711 and retrurns it
    pub fn readout(&mut self) -> Result<i32, E>
    {
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high.
        // Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let mut txrx: [u8; 1] = [0];

        self.spi.transfer(&mut txrx)?;

        while txrx[0] == 0xFF                      // as soon as a single bit is low data is ready
        {
            // sleep for 1 millisecond which is 1/100 of the conversion period to grab the data while it's hot
            sleep(Duration::from_millis(1));
            txrx[0] = 0;
            self.spi.transfer(&mut txrx)?;                                     // and check again
        }

        // the read has the same length as the write.
        // MOSI provides clock to the HX711's shift register (binary 1010...)
        // clock is 10 the buffer needs to be double the size of the 4 bytes we want to read
        let mut buffer: [u8; 8] = [0b10101010, 0b10101010, 0b10101010, 0b10101010,
                                   0b10101010, 0b10101010, self.mode as u8, 0];

        self.spi.transfer(&mut buffer)?;
        // value should be in range 0x800000 - 0x7fffff according to datasheet

        let res = decode_output(&buffer);

        Ok(res)
    }

    pub fn reset(&mut self) -> Result<(), E>
    {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs,
        // HX711 enters power down mode.
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // speed is the raw SPI speed -> half bits per second

        // max SPI clock frequency should be 5 MHz to satisfy the 0.2 us limit for the pulse length
        // we have to output more than 300 bytes to keep the line for at leas 60 us high

        let buffer : [u8; 301] = [0xFF; 301];

        self.spi.write(& buffer)?;

        Ok(())
    }

    pub fn change_mode(&mut self, m: HX711Mode) -> Result<(), E>
    {
        self.mode = m;

        Ok(())
    }
    /*
    pub fn power_down()
    {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
    }
    */
}

#[bitmatch]
fn decode_output(buffer: &[u8;8]) -> i32
{
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
