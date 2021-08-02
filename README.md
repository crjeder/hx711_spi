# hx711_spi

[![Crate](https://img.shields.io/crates/v/hx711_spi?style=plastic)](https://crates.io/crates/hx711_spi)
![License](https://img.shields.io/crates/l/hx711_spi?style=plastic)
[![API](https://docs.rs/hx711_spi/badge.svg)](https://docs.rs/hx711_spi)
![Docs](https://img.shields.io/docsrs/hx711_spi?style=plastic)
![LOC](https://img.shields.io/tokei/lines/github/crjeder/hx711_spi?style=plastic)
![Maintained](https://img.shields.io/maintenance/yes/2021?style=plastic)
![GitHub Repo stars](https://img.shields.io/github/stars/crjeder/hx711_spi?style=plastic)
![Crates.io](https://img.shields.io/crates/d/hx711_spi?style=plastic)
[![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/hx711_spi.svg)](https://web.crev.dev/rust-reviews/crate/hx711_spi/)


This is a platform agnostic driver to interface with the HX711 load cell IC. It uses SPI instad of bit banging.
This `[no_std]` driver is built using [`embedded-hal`][2] traits.

## Why did I write another HX711 driver?
In multi-user / multi-tasking environments bit banging is not reliable. SPI on the other hand handles the timing with hardware support and is not influenced by other processes.

## Usage
Use an embedded-hal implementation to get SPI and Delay.
HX711 does not use SCLK, instead it is provided by the driver using SDI. Make sure
that HX711 is the only device on the bus since it does not implemnt CS.
Connect the SDO to the PD_SCK and SDI to DOUT of the HX711. SPI clock frequency
has to be between 20 kHz and 5 MHz.

## Example
This is just a code snplet to show how the driver is used. A full example is in
['./examples'][https://github.com/crjeder/hx711_spi/blob/next/examples/src/main.rs]

```rust
    let mut hx711 = Hx711::new(spi, Delay::new());

	  hx711.reset()?;
    let v = block!(hx711.read())?;
 	  println!("value = {}", v);
```

## What works
(tested on Raspberry Pi)

  - Reading results
  - Setting the mode (gain and channel)

No scales functions (like tare weight and calibration) are implemented because I feel that's not part of a device driver.

## TODO

  - [ ] Test on more platforms
  - [X] Power down (functions exist just for compatibility. Implementation is not possible with SPI)
  - [X] Reset
  - [X] `[no_std]`
  - [ ] make it re-entrant / thread safe


It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev)
to verify the trustworthiness of each of your dependencies, including this one.



## Feedback
All kind of feedback is welcome. If you have questions or problems, please post them on the issue tracker
This is literally the first code I ever wrote in rust. I am still learning. So please be patient, it might take me some time to fix a bug. I may have to break my knowledge sound-barrier.
If you have tested on another platform I'd like to hear about that, too!

# References

  - [datasheet][1]

[1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf

  - [embedded-hal][2]

[2]: https://github.com/rust-embedded/embedded-hal

## License

Licensed under either of

  - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
  - MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
