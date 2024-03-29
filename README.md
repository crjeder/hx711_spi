# hx711_spi

[![Crate](https://img.shields.io/crates/v/hx711_spi?style=plastic)](https://crates.io/crates/hx711_spi)
![License](https://img.shields.io/crates/l/hx711_spi?style=plastic)
![GitHub branch checks state](https://img.shields.io/github/checks-status/crjeder/hx711_spi/release?style=plastic)
<!--![Docs](https://img.shields.io/docsrs/hx711_spi?style=plastic)-->
<!--![LOC](https://img.shields.io/tokei/lines/github/crjeder/hx711_spi?style=plastic)-->
![Maintained](https://img.shields.io/maintenance/yes/2023?style=plastic)
[![dependency status](https://deps.rs/repo/github/crjeder/hx711_spi/status.svg)](https://deps.rs/repo/github/crjeder/hx711_spi)
![GitHub Repo stars](https://img.shields.io/github/stars/crjeder/hx711_spi?style=plastic)
![Crates.io](https://img.shields.io/crates/d/hx711_spi?style=plastic)
<!-- [![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/hx711_spi_bb.png)](https://web.crev.dev/rust-reviews/crate/hx711_spi/)-->

This is a platform agnostic driver to interface with the HX711 load cell IC. It uses SPI instead of bit banging.
This `[no_std]` driver is built using [`embedded-hal`][2] traits.
It is developed on Raspberry PI and reported to work on STM32 and ESP32.
It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev)
to verify the trustworthiness of each of your dependencies, including this one.

## Why did I write another HX711 driver?
In multi-user / multi-tasking environments bit banging is not reliable. SPI on the other hand handles the timing with hardware support and is not influenced by other processes.

## Usage
Note: I'm using the reddefined SPI signal names (['see sparkfun's Resolution'][3]).

Use an embedded-hal implementation to get SPI.
HX711 does not use SCLK, instead clock is provided by the driver using SDO. Make sure
that HX711 is the only device on the bus since it does not implemnt CS (Chip Select).
Connect the SDO to the PD_SCK and SDI to DOUT of the HX711. SPI clock frequency
has to be between 20 kHz and 5 MHz.
Since the SPI clock is not used, SPI mode 0 or mode 1 should work. You need
test which one gives the best results for you.
The library assumes that SDO signal is idle low. If this is not the case you have to use extra hardware to pull it low. In this case you should use the ```[invert-sdo]``` feature to send the correct signals to the hx711.

No scales functions (like tare weight and calibration) are implemented because I feel that's not part of a device driver.
Power down functions exist just for compatibility. Implementation is not possible with this (ab-) use of SPI since the CPU / MPU would need to constantly send on the bus to power donwn the HX711. This would totally defy the purpose.

## TODO

- Test on more platforms (HALs)
  - [x] Rasperry Pi
 	- [x] STM32
	 - [x] ESP32
  - [x] nrf52840  
	 - [ ] RP2040
 	- [ ] Teensy
- [X] Power Save (functions exist just for compatibility. Implementation is not possible with SPI)
- [X] Reset
- [X] `[no_std]`
- [X] make it re-entrant / thread safe  
- [ ] validate against other libraries (bit banging, python, ..) 
- [ ] async
- [ ] use emedded HAL v1

## Examples
### Raspberry PI
[<img src="examples/hx711_spi_bb.png" width="300">](examples/hx711_spi.fzz)
```text
// embedded_hal implementation
use rppal::{spi::{Spi, Bus, SlaveSelect, Mode, Error},hal::Delay};

use hx711_spi::Hx711;
use nb::block;

// minimal example
fn main() -> Result<(), Error>
{
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let mut hx711 = Hx711::new(spi);

	  hx711.reset()?;
    let v = block!(hx711.read())?;
 	  println!("value = {}", v);

    Ok(())
}
```

### STM32F1
An example stm32f103 (blue pill) initialization (note mode 1).

```text
    use stm32f1xx_hal::time::U32Ext;
    use cortex_m_rt::entry;
    use stm32f1xx_hal::{pac, prelude::*,
      spi::{Mode, Phase, Polarity, Spi}, };

    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let hx711_spi_pins = (
        gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh),
        gpiob.pb14.into_floating_input(&mut gpiob.crh),
        gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh),
    );
    let hx711_spi = spi::Spi::spi2(device.SPI2, hx711_spi_pins, spi::MODE_1, 1.mhz(), clocks);
    let mut hx711_sensor = Hx711::new(hx711_spi);
    hx711_sensor.reset().unwrap();
    hx711_sensor.set_mode(hx711_spi::Mode::ChAGain128).unwrap(); // x128 works up to +-20mV
```

## Roadmap
1.0 Will implement the ```embedded_hal::adc::OneShot``` once it is finalized

## Feedback
All kind of feedback is welcome. If you have questions or problems, please post them on the issue tracker
This is literally the first code I ever wrote in rust. I am still learning. So please be patient, it might take me some time to fix a bug. I may have to break my knowledge sound-barrier.
If you have tested on another platform I'd like to hear about that, too!

Big thanx to ['jbit'](https://github.com/jbit) for clearing the question about thread safety
and ['anddreyk0'](https://github.com/andreyk0) for testing on STM32 and both for debugging!


# References

  - [datasheet][1]

[1]: https://cdn.sparkfun.com/datasheets/Sensors/ForceFlex/hx711_english.pdf

  - [embedded-hal][2]

[2]: https://github.com/rust-embedded/embedded-hal

  - [spi_signal_names][3]

[3]: https://www.sparkfun.com/spi_signal_names

## License

Licensed under either of

  - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [here](http://www.apache.org/licenses/LICENSE-2.0))
  - MIT license ([LICENSE-MIT](LICENSE-MIT) or [here](http://opensource.org/licenses/MIT))

at your option.
