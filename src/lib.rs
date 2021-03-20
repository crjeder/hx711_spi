//! A (soon to be) platform agnostic driver to interface with the HX711 load cell IC.
//!
//! This driver is built using [`embedded-hal`] traits

// #![no_std]

use std::error::Error;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};

// constants from rppal documentation / example
// const WRITE: u8 = 0b0010; // Write data, starting at the selected address.
// const READ: u8 = 0b0011; // Read data, starting at the selected address.
// const RDSR: u8 = 0b0101; // Read the STATUS register.
const WREN: u8 = 0b0110; // Set the write enable latch (enable write operations).
// const WIP: u8 = 1; // Write-In-Process bit mask for the STATUS register.

const SAMPLERATE: u8 = 10; // most boards are fixed to 10 SPS change if your hardware differs

/// The HX711 has two chanels: A for the load cell and B for AD conversion of other signals.
/// This three modes selecte the chips behaviour:
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum HX711Mode {
    /// Convet chanel A with a gain factor of 128
    ChAGain128 = 0x80,
    /// Convert chanel B with a gain factor of 64
    ChBGain32 = 0xC0,
    /// Convert chanel B with a gain factor of 32
    ChBGain64 = 0xE0,
}


/// Represents an instance of a HX711 device
pub struct Hx711
{
    spi: Spi,
    mode: HX711Mode
}

impl Hx711
{
    /// opens a connection to a HX711 on a specified SPI
    pub fn new(bus: Bus) -> Result<Hx711, Box<dyn Error>>
    {
        let dev = Spi::new(bus, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;

        Ok
        (
            Hx711
            {
                spi: dev,
                mode: HX711Mode::ChAGain128
            }
        )
    }

    /// reads a value from the HX711 and retrurns it
    pub fn readout(&mut self) -> Result<i32, Box<dyn Error>>
    {
        // the read has the same length as the write.
        // MOSI provides clock to the HX711's shift register (binary 1010...)
        // clock is 10 the buffer needs to be double the size of the 4 bytes we want to read
        let tx_buf: [u8; 8] = [0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, self.mode as u8, 0];
        let mut rx_buf: [u8; 8] = [0; 8];

        self.spi.write(&[WREN])?;                               // write enable

        let transfer = Segment::new(&mut rx_buf, &tx_buf);
        self.spi.transfer_segments(&[transfer])?;
        // use every second bit from the buffer
        for byte = [0..7]
        {
            for bit = [0..7]
            {
                //
            }
        }
        // let result = i32::from_be_bytes(rx_buf) / 0x100;       // upper 24 bits only

        Ok(result)                                              // return value
    }
}
