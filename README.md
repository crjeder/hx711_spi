# hx711_spi
This is a platform agnostic driver to interface with the HX711 load cell IC. It uses SPI instad of bit banging.
This driver is built using [`embedded-hal`][2] traits.

## Why did I write another HX711 driver?
In multi-user / multi-tasking environments bit banging is not reliable. SPI on the other hand handles the timing with hardware support and is not influenced by other processes.

## Usage
Use embedded-hal implementation to get SPI. HX711 does not use CS and SCLK. Make sure that it
is the only device on the bus. Connect the SDO to the PD_SCK and SDI to DOUT of the HX711.

```rust
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use hx711_spi::{Hx711, HX711Mode};

let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();

// to create sensor with default configuration:
let mut scale = Hx711(spi);

// start measurements
let mut value = scale.readout().unwrap();
```

# References

- [Datasheet][1]

[1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf

- [embedded-hal][2]

[2]: https://github.com/rust-embedded/embedded-hal

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
