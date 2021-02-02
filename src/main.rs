extern crate spidev;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};

/// The HX711 can run in three modes:
#[derive(Copy, Clone)]
pub enum Mode {
    /// Chanel A with factor 128 gain
    ChAGain128 = 0x80,
    /// Chanel B with factor 64 gain
    ChBGain32 = 0xC0,
    /// Chanel B with factor 32 gain
    ChBGain64 = 0xE0,
}

pub struct Hx711
{
    device: Spidev,
    mode: Mode
}

impl Hx711
{
    pub fn new(path: &String) -> Hx711
    {
        let options = SpidevOptions::new()
             .bits_per_word(32)
             .max_speed_hz(10000)
             .mode(SpiModeFlags::SPI_MODE_0)
             .build();
        let mut dev = Spidev::open(path).unwrap();
        dev.configure(&options).unwrap();

        Hx711
        {
            device: dev,
            mode: Mode::ChAGain128
        }
    }

    pub fn readout(&self, nr_values: u8) -> i32
    {
        // "write" transfers are also reads at the same time with
        // the read having the same length as the write
        let tx_buf = [0xaa, 0xaa, 0xaa, self.mode as u8];
        let mut rx_buf = [0; 4];
        let mut values: Vec<i32> = Vec::new();
        let mut result: i32 = 0;

        for _i in 1..=nr_values
        {
            let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
            self.device.transfer(&mut transfer).unwrap();
            println!("{:?}", rx_buf);
            values.push(i32::from_be_bytes(rx_buf));
        }

        // arithmetic average over the values
        for element in values.iter()
        {
            result = result + element;
        }
        result / nr_values as i32         // return value
    }
}

fn main() {
    println!("Hello, world!");
}
