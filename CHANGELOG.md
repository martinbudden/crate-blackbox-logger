# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Releases of the form `0.1.n` do not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
that is each release may contain incompatible API changes.

Once the API has stabilized this project will adopt semantic versioning, the first release to do so will be `0.2.0`.

## [Unreleased]

### Added

### Changed

- Improved writing of `log_main_fields_header` name.
- Renamed `StateMachine` to `LoggerState`.
- Renamed `FieldHeader` to `FieldHeaderIndex`.
- Moved setting `logged_any_frames` into `log_iteration`.
- Added `Blackbox` prefix to many exported structs.
- Renamed `SliceEncoder::encoder` to `write_tag_3s32`.
- Updated slow frame and gps frame field definitions.
- `main_data` is now indexed rather than copied.
- Write first slow frame near start of log.

### Removed

- `MockSdCard`.
- `GyroPidMessage`
- `SetpointMessage`

### Deprecated

### Fixed

### Security

## [0.1.7] - 2026-06-05

### Changed

- Updated to vqm 0.1.11.
- use `u32` for feature flags in `Logger`.

### Removed

- `features.rs`

## [0.1.6] - 2026-05-29

### Added

- `CONTRIBUTING.md`.

### Changed

- `README.md`

## [0.1.5] - 2026-05-29

### Added

- Support for `sequential-storage`.

### Changed

- Updated to vqm 0.1.8.
- Improved handling of `gps` feature.

### Removed

- `katex-header.html`.

## [0.1.4] - 2026-05-23

### Changed

- Updated to vqm 0.1.8.
- made `serde` an optional feature.

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
