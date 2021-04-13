# hx711_spi
A platform agnostic driver to interface with the HX711 load cell IC.
This is (will be) a library for the hx711 load cell chip. It uses spi instad of bit banging.

//!
//!
//! This driver is built using [`embedded-hal`] traits
//! [embedded-hal]: https://docs.rs/embedded-hal
//!
//! # Usage
//!
//! Use embedded-hal implementation to get SPI. HX711 does not use CS and SCLK. Make sure that it
//! is the only device on the bus. Connect the SDO to the PD_SCK and SDI to DOUT of the HX711.
//!
//! // to create sensor with default configuration:
//! let mut scale = Hx711(SPI);
//!
//! // start measurements
//! let mut value = scale.readout();
//!
//! # References
//!
//! - [Datasheet][1]
//!
//! [1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf
//!
//! - [`embedded-hal`][2]
//!
//! [2]: https://github.com/rust-embedded/embedded-hal
//!
//!
