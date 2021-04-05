// use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use std::{thread, time};

use hx711_spi::Hx711;

fn main()
{
	// just testing
	let buffer: [u8; 8] = [0b1010_1010; 8];
	let mut raw: [u8; 4] = [0; 4];

    for bit in 8..31
    {
        raw[3 - (bit / 8)] &= (buffer[8 - ((bit * 2) / 8)] >> (bit *2) & 1) << (bit % 8);
    }

	let r = i32::from_be_bytes(raw) / 0x100;

	assert_eq!(r, -1);
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let mut test = Hx711::new(spi).unwrap();
    // test.spi.configure()
	loop
	{
        let v = test.readout().unwrap();
		println!("value = {}", v);
		thread::sleep(time::Duration::from_millis(100));
	}
}
