# hx711_spi
This is a platform agnostic driver to interface with the HX711 load cell IC. It uses SPI instad of bit banging.
This driver is built using [`embedded-hal`] traits [embedded-hal]: https://docs.rs/embedded-hal

## Why yet an other HX711 driver?
In mult-iuser / multi-tasking environments bit banging is not reliable. SPI on the other hand handles the timing with hardware support and is not influenced by other processes.

## Usage
Use embedded-hal implementation to get SPI. HX711 does not use CS and SCLK. Make sure that it
is the only device on the bus. Connect the SDO to the PD_SCK and SDI to DOUT of the HX711.

```rust
// to create sensor with default configuration:
let mut scale = Hx711(SPI);

// start measurements
let mut value = scale.readout();
```

# References

- [Datasheet][1]

[1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf

- [`embedded-hal`][2]

[2]: https://github.com/rust-embedded/embedded-hal

## License

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
