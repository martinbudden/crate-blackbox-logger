# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Releases of the form `0.1.n` do not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
that is each release may contain incompatible API changes.

Once the API has stabilized this project will adopt semantic versioning, the first release to do so will be `0.2.0`.

## [Unreleased]

### Added

### Changed

- made `serde` an optional feature.

### Removed

### Deprecated

### Fixed

### Security

## [0.1.4] - 2026-05-23

### Changed

- Updated to vqm 0.1.8.
- made `serde` and optional feature.

## [0.1.3] - 2026-05-16

### Added

- `log_gps_h_fields_header` and `log_gps_g_fields_header`.

### Changed

- Updated to vqm 0.1.5.
- Changed constructors to be `const` where possible.
- Updated documentation.
- Improved event logging.

## [0.1.2] - 2026-05-10

### Changed

- Fixed `MainFieldDefinition` for "eRPM" fields.
- Fixed handling of disabled fields in `Logger::init`.

## [0.1.1] - 2026-05-06

### Changed

- Updated to vqm version 0.1.3.
- Made `BlackboxConfig::new` const.

## [0.1.0] - 2026-04-25

Initial release.
