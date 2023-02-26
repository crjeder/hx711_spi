// embedded_hal implementation
use rppal::spi::{Bus, Error, Mode, SlaveSelect, Spi};

use hx711_spi::Hx711;
use nb::block;

// minimal example
fn main() -> Result<(), Error> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode1)?;
    let mut hx711 = Hx711::new(spi));

    hx711.reset()?;
    let v = block!(hx711.read())?;
    println!("value = {}", v);

    Ok(())
}
