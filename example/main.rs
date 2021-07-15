use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use embedded_hal::blocking::delay::DelayMs;
use rppal::hal::Delay;

use hx711_spi::Hx711;

fn main()
{
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let mut test = Hx711::new(spi, Delay::new()).unwrap();
    // test.spi.configure()

	test.reset().unwrap();

    let mut delay = Delay::new();
	loop
	{
        let v = block!(test.retrieve().unwrap());
		println!("value = {}", v);
		delay.delay_ms(1u8);
	}
}
