# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Changed
- Add a terminal screenshot to the README.
- Replace generated README screenshot with a real capture.
- Document crates.io install as the primary README path.

## [0.1.0] - 2026-06-13

### Added
- Initial release: colorful terminal system monitor (CPU, memory, swap, process table)
- Process sorting, filtering, kill menu, tree view, and help overlay
- Makefile for build, install, test, and lint workflows

### Changed
- Collapse CPU panel to a single average row by default (`c` expands per-core bars)
- Polish meters, status bar, and help overlay with a richer color theme
- Right-align memory and swap used/total labels in meter suffixes

### Fixed
- README clone URL points at the GitHub repository
- CPU meter closing bracket layout when percentage is shown inside the bar

### Documentation
- README footprint notes for release binary size and idle RSS

<!-- next-url -->
[Unreleased]: https://github.com/cesarferreira/toppy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cesarferreira/toppy/releases/tag/v0.1.0
