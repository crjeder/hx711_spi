use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};
use std::{thread, time};

use hx711_spi::Hx711;

fn main()
{
	// just testing
	let buffer[u8] = [0b1010_10101, 0b1010_10101, 0b1010_10101, 0b1010_10101, 0b1010_10101, 0b1010_10101, 0b1010_10101, 0b1010_10101];
	let mut raw: [u8; 4] = [0; 4];

    for bit in 8..31
    {
        raw[3 - (bit / 8)] &= (buffer[8 - ((bit * 2) / 8)] >> (bit *2) & 1) << (bit % 8);
    }
	
	let r = i32::from_be_bytes(raw) / 0x100;
	
	assert_eq!(r, 0xFFFF_FFFF);
	
    let mut test = Hx711::new(rppal::spi::Bus::Spi0).unwrap();
	loop
	{
        let v = test.readout().unwrap();
		println!("value = {}", v);
		thread::sleep(time::Duration::from_millis(100));
	}
}
