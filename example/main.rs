use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use std::{thread, time};

use hx711_spi::Hx711;

#![allow(unused)]
#![doc(html_root_url = "https://docs.rs/hx711_spi/0.2.1")]
fn main()
{
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let mut test = Hx711::new(spi).unwrap();
    // test.spi.configure()

	test.reset().unwrap();

	loop
	{
        let v = test.readout().unwrap();
		println!("value = {}", v);
		thread::sleep(time::Duration::from_millis(100));
	}
}
