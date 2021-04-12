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

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;

use embedded_hal as hal;
use hal::blocking::spi;

// use bitmach to decode the result
use bitmatch::bitmatch;

const SAMPLERATE: u8 = 10; // most boards are fixed to 10 SPS change if your hardware differs

/// The HX711 has two chanels: A for the load cell and B for AD conversion of other signals.
/// This three modes selecte the chips behaviour:
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
    mode: HX711Mode,
    speed: u32
}

impl <SPI, E> Hx711<SPI>
where
    SPI: spi::Transfer<u8, Error=E> + spi::Write<u8, Error=E>
{
    /// opens a connection to a HX711 on a specified SPI
    pub fn new(spi:SPI, s: u32) -> Result<Self, E>
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
                speed: s
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

        // variant with sleep
        print!("{:?}, ", SystemTime::now().duration_since(UNIX_EPOCH));
        self.spi.transfer(&mut txrx)?;
        println!("ready?: {}", txrx[0]);

        while txrx[0] == 0xFF                      // as soon as a single bit is low data is ready
        {
            // sleep for a 1/10 of the conversion period to grab the data while it's hot
            sleep(Duration::from_millis((SAMPLERATE / 1000).into()));
            txrx[0] = 0;
            self.spi.transfer(&mut txrx)?;                                     // and check again
            // println!("{:?}, ready?: {}", SystemTime::now().duration_since(UNIX_EPOCH), txrx[0]);
        }
        println!("{:?}, ready?: {}", SystemTime::now().duration_since(UNIX_EPOCH), txrx[0]);
        // the read has the same length as the write.
        // MOSI provides clock to the HX711's shift register (binary 1010...)
        // clock is 10 the buffer needs to be double the size of the 4 bytes we want to read
        let mut buffer: [u8; 8] = [0b10101010, 0b10101010, 0b10101010, 0b10101010,
                                   0b10101010, 0b10101010, self.mode as u8, 0];

        self.spi.transfer(&mut buffer)?;
        // value should be in range 0x800000 - 0x7fffff according to datasheet

        println!("buffer = {:?}", buffer);
        let res = decode_output(&buffer);
        println!("result = {:b}", res);

        Ok(res)
    }


    pub fn reset(&mut self) -> Result<(), E>
    {
        // when PD_SCK pin changes from low to high and stays at high for longer than 60µs,
        // HX711 enters power down mode.
        // When PD_SCK returns to low, chip will reset and enter normal operation mode.
        // speed is the raw SPI speed -> half bits per second
        let n = (60.0e-6 * (self.speed as f32 / 2.0)).ceil() as u32;
        let buffer : [u8; 1] = [0xFF];

        for _i in 0..n
        {
            self.spi.write(& buffer)?;
        }
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
