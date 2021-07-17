# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- reset()
-- get_mode()
-- set_mode()
-- enable()
-- disable()
- [no_std]
- nb for read()
- CHANGELOG
### Changed
- Due to the change to [no_std] the interface of new() had to change: it needs a Delay parameter now.
- Due to nb the return type of read() has changed to nb::Result

## [0.2.1] - 2021-04-14
Initial release

[0.3.0]: https://github.com/crjeder/hx711_spi/releases/tag/0.3.0
[0.2.1]: https://github.com/crjeder/hx711_spi/releases/tag/0.2.1
