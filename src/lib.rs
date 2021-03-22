//! A (soon to be) platform agnostic driver to interface with the HX711 load cell IC.
//!
//! This driver is built using [`embedded-hal`] traits (not yet, obviously)

// #![no_std]

use std::error::Error;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};
use std::time::Duration;
use std::thread::sleep;

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
    ChAGain128 = 0b10000000,
    /// Convert chanel B with a gain factor of 64
    ChBGain32 = 0b10100000,
    /// Convert chanel A with a gain factor of 32
    ChBGain64 = 0b10101000,
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
        // datasheet specifies PD_SCK high time and PD_SCK low time to be in the 0.2 to 50 us range 
        // therefore bus speed is 5 MHz to 20 kHz. 1 MHz seems to be a good choice
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
        self.spi.write(&[WREN])?;                               // write enable
        
        // check if data is ready
        // When output data is not ready for retrieval, digital output pin DOUT is high. Serial clock input PD_SCK should be low. When DOUT goes
        // to low, it indicates data is ready for retrieval.
        let tx: u8 = 0;
        let mut rx: u8 = 0;
                        
        // variant with error
        /*
        let check = Segment::new(&mut rx; &tx);
        self.spi.transfer_segments(&[check])?;
        
        if rx == 0
        {
            Error::new(ErrorKind::Other, "Output not ready while read") // hopefully that's inline with the best practices
        }
        */
      
        // variant with sleep
        let check = Segment::new(&mut rx, &tx);
        
        self.spi.transfer_segments(&[check])?;
        
        while rx == 0
        {
            sleep(Duration::from_millis(((1 / SAMPLERATE) * 1000) / 10).into());   // sleep for a 1/10 of the conversion period to grab the data while it's hot
            self.spi.transfer_segments(&[check])?;                              // and check again      
        }
        
        // the read has the same length as the write.
        // MOSI provides clock to the HX711's shift register (binary 1010...)
        // clock is 10 the buffer needs to be double the size of the 4 bytes we want to read
        let tx_buf: [u8; 8] = [0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, self.mode as u8, 0];
        let mut rx_buf: [u8; 8] = [0; 8];

        let transfer = Segment::new(&mut rx_buf, &tx_buf);
        self.spi.transfer_segments(&[transfer])?;
        
        // now the rx_buffer contains the 2's complement of the reading with every bit doubled.
        // therefore we use every second bit from the buffer
        let result: i32 = 0;
        
        for bit in [0..64].step_by(2)                              // counting in reverse order: bit 0 is MBS skip every second bit
        {   
            result &= rx_buff[bit / 8] << (bit / 2);            // works only for correct endian
        }
        // let result = i32::from_be_bytes(rx_buf) / 0x100;       // upper 24 bits only

        Ok(result)                                              // return value
    }
    
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
}
