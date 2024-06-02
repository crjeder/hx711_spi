// embedded_hal implementation
use rppal::spi::{Bus, Error, Mode, SlaveSelect, Spi};

use hx711_spi::{Hx711, Hx711Error, Mode as HxMode};

// minimal example
fn main() -> Result<(), Hx711Error<Error>> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode1)?;
    let mut hx711 = Hx711::new(spi);

    hx711.reset()?;
    hx711.set_mode(HxMode::ChAGain128)?;
    let v = hx711.read()?;
    println!("value = {:?}", v);

    Ok(())
}
