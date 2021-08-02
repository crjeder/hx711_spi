// embedded_hal implementation
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode, Error},hal::Delay};

use hx711_spi::Hx711;
use nb::block;

// minimal example
fn main() -> Result<(), Error>
{
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let mut hx711 = Hx711::new(spi, Delay::new());

	hx711.reset()?;
    let v = block!(hx711.read())?;
	println!("value = {}", v);

    Ok(())
}
