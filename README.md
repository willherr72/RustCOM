# RustCOM - COM Port Analyzer

A COM port analyzer built with Rust and egui. Provides serial communication tools with a dark-themed GUI.

## Download

Pre-built Windows executables are available on the [Releases](../../releases) page. Download `rustcom.exe` and run it directly — no installation needed.

## Features

- **Auto-detect COM ports** with automatic scanning for new/removed devices
- **ASCII, Hex, and dual view modes** with proper hex dump formatting
- **ASCII and Hex send modes** — type text or raw hex bytes (`AA BB 0D 0A`)
- **Configurable line endings** — None, `\r`, `\n`, `\r\n`
- **DTR/RTS signal control**
- **Data logging** with timestamped entries and file export
- **Regex filtering** on incoming data
- **Auto-reconnect** on connection loss
- **Virtual COM port** creation via com0com (Windows) or socat (Linux)
- **Byte counters** for TX and RX

## Connection Settings

- **Baud Rate**: 300 to 921600
- **Data Bits**: 5, 6, 7, 8
- **Stop Bits**: 1, 2
- **Parity**: None, Even, Odd
- **Flow Control**: None, Software (XON/XOFF), Hardware (RTS/CTS)

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/)
- Windows: [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) with "Desktop development with C++"

### Build and Run

```bash
cargo run --release
```

The compiled exe will be at `target/release/rustcom.exe`.

## Project Structure

```
RustCOM/
├── Cargo.toml
├── build.rs            # Windows icon embedding
├── src/
│   ├── main.rs         # Entry point
│   ├── app.rs          # App struct, constants, display logic
│   ├── serial.rs       # Serial enums, connect/disconnect/send
│   ├── ui.rs           # GUI rendering
│   ├── hex.rs          # Hex formatting and parsing
│   ├── logging.rs      # Data logging and file export
│   └── virtual_com.rs  # Virtual COM port creation
└── README.md
```

## Dependencies

- **eframe** / **egui** — Immediate-mode GUI
- **serialport** — Serial port communication
- **chrono** — Timestamps
- **regex** — Data filtering

## Troubleshooting

- **Port access denied**: Close other apps using the port. On Linux, add yourself to `dialout`: `sudo usermod -a -G dialout $USER`
- **Port not listed**: Click Refresh or enable auto-scan in the Advanced section.

## License

See LICENSE file for details.
