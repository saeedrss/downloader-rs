# downloader-rs

Multi-proxy chunk downloader with ratatui TUI.

## Features

- Chunked parallel downloads via multiple SOCKS5/HTTP proxies
- Best-first proxy selection (tracks avg speed per proxy)
- Adaptive background proxy health checker
- Dynamic workers, timeout (runtime-adjustable via keyboard)
- Pause / Resume
- Qt-like TUI: header, toolbar, tabbed multi-file, log panel, parts grid, proxy list, status bar
- Sequential multi-URL batch
- Resume partial downloads via integrity validation
- Cross-platform (Windows / Linux / macOS)

## Quick Start

```bash
# Rust TUI (from downloader-rs/)
cargo run --release -- --url <URL> --proxies proxies.txt
```

### Pre-built binary

```bash
./target/release/downloader --url <URL> --proxies proxies.txt
```

## CLI Options

| Argument             | Default                      | Description                                     |
|----------------------|------------------------------|-------------------------------------------------|
| `--url`              | —                            | URL(s) to download (comma-separated)            |
| `--proxies`          | —                            | Proxy file path or comma-separated list         |
| `--size`             | `5`                          | Chunk size in MB                                |
| `--connections`      | `5`                          | Max concurrent downloads                        |
| `--timeout`          | auto `max(30, size*1024/56)` | Total request timeout (seconds)                 |
| `--connect-timeout`  | `20`                         | Connect timeout (seconds)                       |
| `--read-timeout`     | `35`                         | Read timeout (seconds)                          |
| `--output`           | —                            | Output file path (single URL only)              |
| `--output-dir`       | `.`                          | Output directory                                |
| `--temp-dir`         | `temp`                       | Temporary directory for parts                   |
| `--useproxyformulticon` | —                         | Allow one proxy for multiple connections        |

## Keyboard Controls

| Key           | Action                  |
|---------------|-------------------------|
| `P`           | Pause / Resume          |
| `+` / `=`     | Increase workers        |
| `-` / `_`     | Decrease workers (min 1)|
| `]`           | Increase timeout +10s   |
| `[`           | Decrease timeout −10s (min 10s) |
| `←` `→`       | Switch tabs             |
| `q` / `Esc`   | Quit TUI                |

## Build

```bash
cargo build --release
```

### Dependencies

`tokio`, `reqwest` (socks), `clap`, `ratatui`, `crossterm`, `crossbeam-channel`, `futures`

## Architecture

```
src/
├── main.rs       — CLI args, channel setup, spawns downloads + TUI
├── download.rs   — chunk download, proxy filtering, health checker
├── state.rs      — DynState, DynamicSemaphore, ProxyStats
└── tui.rs        — Ratatui TUI rendering and event loop
```

- Downloads run on a `tokio` runtime in a background thread
- TUI runs on the main thread via `ratatui` (crossterm backend)
- A `crossbeam-channel` bridges download events → TUI
- `DynState` (`Arc<Atomic…>`) enables cross-thread pause / workers / timeout

## License

MIT
