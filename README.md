# toppy

A colorful terminal system monitor inspired by htop and btop. Built with Rust and [ratatui](https://ratatui.rs).

## Features

- Per-core CPU bars with utilization coloring
- RAM and swap usage bars with KB/MB/GB labels
- Process table focused on **PID**, **CPU%**, and **Command**
- Full keyboard control: sort, filter, tree view, kill, help

## Requirements

- Rust **1.85+** (edition 2024)
- macOS or Linux

## Install

```bash
cargo install --path .
```

This installs the `toppy` binary to your Cargo bin directory (usually `~/.cargo/bin`).

## Usage

```bash
toppy
toppy --refresh-rate 500
```

Press `q` to quit.

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `↑` / `↓` | Move selection |
| `PgUp` / `PgDn` | Page up/down |
| `Home` / `End` | First/last row |
| `P` / `C` / `M` / `T` | Sort by PID / CPU / MEM / Command |
| `/` | Filter by command or PID |
| `Esc` | Clear filter / close overlay |
| `t` | Toggle process tree view |
| `→` / `←` / `Enter` | Expand/collapse tree node |
| `k` / `F9` | Kill selected process |
| `?` / `F1` | Help overlay |
| `r` | Force refresh |

In the kill menu: `1` = SIGTERM, `2` = SIGKILL.

## License

MIT
