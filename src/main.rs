use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};

use hx711_spi::Hx711;

fn main()
{
        let mut test = Hx711::new(rppal::spi::Bus::Spi0).unwrap();
        let v = test.readout().unwrap();
        println!("value = {}", v);
}
