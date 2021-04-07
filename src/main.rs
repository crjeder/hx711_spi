// use rppal::spi::{Bus, Mode, SlaveSelect, Spi, Segment};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use std::{thread, time};
use bitmatch::bitmatch;

use hx711_spi::Hx711;

fn main()
{
	// just testing
	let buffer: [u8; 8] = [0b1010_1010; 8];
	let r = decode(&buffer);

	println!("value = {}", r / 0x100);

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

#[bitmatch]
fn decode(buffer: &[u8;8]) -> i32
{
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

	i32::from_be_bytes(raw)
}
