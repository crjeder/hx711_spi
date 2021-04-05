//! HX711 embedded-hal SPI driver crate
//!
//! A platform agnostic driver to interface with the HX711 load cell IC.
//!
//! This driver is built using [`embedded-hal`] traits
//! [embedded-hal]: https://docs.rs/embedded-hal
//!
//! # Usage
//!
//! Use embedded-hal implementation to get SPI. HX711 does not use CS and SCLK. Make sure that it
//! is the only device on the bus.
//!
//! // to create sensor with default configuration:
//! let mut scale = Hx711(SPI)
//!
//! // start measurements
//! let mut value = scale.readout();
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

const SAMPLERATE: u8 = 10; // most boards are fixed to 10 SPS change if your hardware differs

/// The HX711 has two chanels: A for the load cell and B for AD conversion of other signals.
/// This three modes selecte the chips behaviour:
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum HX711Mode {
    /// Convet chanel A with a gain factor of 128
    ChAGain128 = 0b10000000,
    /// Convert chanel B with a gain factor of 64
    ChBGain32 = 0b10100000,
    /// Convert chanel A with a gain factor of 32
    ChBGain64 = 0b10101000,
}

/// Represents an instance of a HX711 device
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
                mode: HX711Mode::ChAGain128
            }
        )
    }

    /// reads a value from the HX711 and retrurns it
    pub fn readout(&mut self) -> Result<i32, E>
    {
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high. Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let mut txrx: [u8; 1] = [0];

        // variant with sleep
        self.spi.transfer(&mut txrx)?;

        while txrx[0] == 0
        {
            // sleep for a 1/10 of the conversion period to grab the data while it's hot
            sleep(Duration::from_millis((100 / SAMPLERATE).into()));
            self.spi.transfer(&mut txrx)?;                                     // and check again
        }
		
        // the read has the same length as the write.
        // MOSI provides clock to the HX711's shift register (binary 1010...)
        // clock is 10 the buffer needs to be double the size of the 4 bytes we want to read
        let mut buffer: [u8; 8] = [0b10101010, 0b10101010, 0b10101010, 0b10101010,
                                   0b10101010, 0b10101010, self.mode as u8, 0];

        self.spi.transfer(&mut buffer)?;

        Ok(decode(buffer[]))
    }
	
    /*
    pub fn reset()
    {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
    }

    pub fn power_down()
    {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs, HX711 enters power down mode
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
    }
    */
	
	// now buffer contains the 2's complement of the reading with every bit doubled
        // since the first byte is the most significant it's big endian
        // we have to extract every second bit from the buffer
        // only the upper 24 (doubled) bits are valid
	
	// now buffer contains the 2's complement of the reading with every bit doubled
    // since the first byte is the most significant it's big endian
    // we have to extract every second bit from the buffer
    // only the upper 24 (doubled) bits are valid

	fn decode(buffer[u8;8]) -> i32
	{
		let mut raw: [u8; 4] = [0; 4];

        for bit in 8..31
        {
            raw[3 - (bit / 8)] &= (buffer[8 - ((bit * 2) / 8)] >> (bit *2) & 1) << (bit % 8);
        }
		
		i32::from_be_bytes(raw) / 0x100				// return value (upper 24 bits)
	}
}
