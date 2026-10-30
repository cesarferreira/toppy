# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added
- btop-style process colors: CPU% and program name colored by CPU load, MEM by memory share, and rows fade with distance from the selection
- `p` toggles the command column between the full command line and the program name (remembered across restarts)
- `f` toggles process-follow mode with a persistent target PID indicator; navigation or a missing target stops following
- Repeatable metrics and PTY footprint measurements, plus process identity, overlay, cache, shortcut, rendering, signal, and terminal regression tests

### Fixed
- Improve selected-row contrast and extend the highlight across the full process panel width in table and tree views
- Linux: list processes instead of every thread
- Keep rows and viewport stationary by default while rankings change, and pin kill confirmations to the original PID and start time independently of follow mode
- Re-read process identity before signaling instead of trusting the last metrics snapshot
- Preserve table/tree view under help, filtering, and kill overlays; tree filtering includes matching ancestors
- Refresh metrics on a deadline even under continuous input, and restore terminal state after partial setup failures
- Revalidate cached command metadata against process identity and current arguments
- Support the documented F1 help shortcut, ignore key-release events, and page by the visible row count
- Show F1/F9 aliases in help and fit the help content on standard-height terminals
- Keep action feedback visible across scheduled refreshes and validate expansion identities without quadratic searches

### Documentation
- Describe lowercase `c`/`p`, saved preference paths, overlay behavior, signal safety limits, and measurement commands

### Changed
- Install the executable as `tp` instead of `toppy` (the package remains `toppy`).

## [0.4.0] - 2026-07-25

### Added
- Remember CPU per-core view across restarts

## [0.3.0] - 2026-06-16

### Changed
- Enable LTO, single codegen unit, strip, panic=abort
- Cache command strings + lowercase keys, narrow startup refresh
- Drain child lists into TreeNode instead of cloning them
- Cache the flattened tree on App and only rebuild in tree mode
- Render meter bars straight into the ratatui Buffer
- Tighter width math in row renderers + truncate_to_width fast path
- Handle input before redrawing, and coalesce queued events

### Documentation
- Refresh footprint numbers after the perf stack

## [0.2.0] - 2026-06-13

### Changed
- Add a terminal screenshot to the README.
- Replace generated README screenshot with a real capture.
- Document crates.io install as the primary README path.
- Release scripts
- Refine project description and add crates.io repository metadata.
- Updated readme

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
[Unreleased]: https://github.com/cesarferreira/toppy/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/cesarferreira/toppy/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/cesarferreira/toppy/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/cesarferreira/toppy/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cesarferreira/toppy/releases/tag/v0.1.0
