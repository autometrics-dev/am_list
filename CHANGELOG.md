# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Fixed

- [Rust] The struct name is now part of the module path for detected methods
- [Rust] Modules defined within a source file are properly detected, and part
  of the module path for detected methods

## [Version 0.2.0] – 2023-06-07

### Added

### Changed

- The command to list all the function names is now a subcommand called 'list'. The
  change is done to accomodate for different subcommands in the future.
- The output of the `list` command is now in JSON, to ease consumption for other
  programs

### Deprecated

### Removed

### Fixed

### Security

## [Version 0.1.0] – 2023-05-29

### Added

- Support for parsing Rust and Go projects

### Changed

### Deprecated

### Removed

### Fixed

### Security

<!-- next-url -->
[Unreleased]: https://github.com/gagbo/am_list/compare/v0.2.0...HEAD
[Version 0.2.0]: https://github.com/gagbo/am_list/compare/v0.1.0...v0.2.0
[Version 0.1.0]: https://github.com/gagbo/am_list/releases/tag/v0.1.0
