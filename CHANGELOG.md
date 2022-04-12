# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2022-04-12
Bugfix release

## [0.3.0] - 2021-07-17
### Added
- reset()
- mode()
- set_mode()
- enable() (unimplemented see documentation)
- disable() (unimplemented see documentation)
- compatibility to [Hx711 crate](https://github.com/jonas-hagen/hx711)
-- get_mode()
-- set_mode()
-- enable()
-- disable()
-- retrieve() as an alias to read()
- [no_std]
- nb for read()
- CHANGELOG
### Changed
- Due to the change to [no_std] the interface of new() had to change: it needs a Delay parameter now.
- Due to nb the return type of read() has changed to nb::Result
- readout() was changed to read()
- Due to nb the return type of read() / readout() has changed to nb::Result
- readout() was changed to read()
- readout() as alias to read()

## [0.2.1] - 2021-04-14
Initial release

[Unreleased]: https://github.com/crjeder/hx711_spi
[0.3.0]: https://github.com/crjeder/hx711_spi/releases/tag/0.3.0
[0.2.1]: https://github.com/crjeder/hx711_spi/releases/tag/0.2.1
