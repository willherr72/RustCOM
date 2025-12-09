# RustCOM - Professional COM Port Analyzer

A modern, feature-rich COM port analyzer built with Rust. RustCOM provides professional-grade serial communication tools with an intuitive dark-themed GUI.

## ✨ Features

### 🔌 Core Functionality
- **Auto-detect COM ports** - Automatically discovers all serial ports on your system
- **Full connection control** - Complete configuration of serial parameters
- **Real-time communication** - Live data transmission and reception
- **Multiple baud rates** - Supports 300 to 921600 baud

### 📊 Advanced Views
- **ASCII View** - Standard text display
- **Hexadecimal View** - Raw hex dump with offsets and ASCII preview
- **Dual View Mode** - See both ASCII and hex simultaneously
- **Auto-scroll** - Automatically follows incoming data
- **Timestamps** - Optional timestamps for received data

### 📡 Signal Control
- **DTR Control** - Data Terminal Ready signal toggle
- **RTS Control** - Request To Send signal toggle
- **Live Signal Monitoring** - Real-time status of control signals

### 📝 Data Logging
- **Session logging** - Record all transmitted and received data
- **Timestamped entries** - Each log entry includes precise timestamps
- **Export to file** - Save logs in human-readable format
- **Buffer capture** - Save current terminal buffer at any time

### 🔍 Advanced Filtering
- **Regex filtering** - Display only data matching specific patterns
- **Real-time filtering** - Applied as data arrives
- **Pattern validation** - Instant feedback on regex syntax

### 🔬 Protocol Analysis
- **Modbus RTU** - Protocol analyzer for Modbus RTU
- **Modbus ASCII** - Protocol analyzer for Modbus ASCII  
- **Custom protocols** - Framework for adding custom analyzers
- **Packet decoding** - Automatic interpretation of protocol frames

### 🔄 Virtual COM Emulator
- **Virtual port pairs** - Create loopback COM port connections
- **Platform support**:
  - Windows: com0com integration
  - Linux: socat-based virtual ports
  - macOS: PTY-based implementation
- **Testing support** - Perfect for testing serial applications

### 📈 Statistics
- **Byte counters** - Track bytes sent and received
- **Connection time** - Monitor session duration
- **Real-time updates** - Live statistics display

### 🔄 Auto-Reconnect
- **Automatic reconnection** - Detects connection loss and reconnects automatically
- **Configurable retry delay** - Set interval between reconnection attempts (500ms - 10s)
- **Connection monitoring** - Detects when devices are unplugged or cables disconnected
- **Visual feedback** - Clear status showing reconnection attempts
- **Event logging** - Terminal logs disconnection and reconnection events
- **Manual control** - Disable auto-reconnect or manually disconnect at any time

## 🎨 User Interface

- **Dark theme** - Easy on the eyes for long sessions
- **Responsive layout** - Adapts to window size
- **Organized panels** - Logical grouping of features
- **Professional styling** - Modern, clean design
- **Status indicators** - Clear visual feedback

## 🛠️ Connection Settings

- **Baud Rate**: 300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600
- **Data Bits**: 5, 6, 7, 8
- **Stop Bits**: 1, 2  
- **Parity**: None, Even, Odd
- **Flow Control**: None, Software (XON/XOFF), Hardware (RTS/CTS)

## 📋 Prerequisites

### Rust Installation

**Windows/Mac/Linux:**
1. Visit [https://rustup.rs/](https://rustup.rs/)
2. Follow the installation instructions for your platform

### Windows-Specific Requirements

On Windows, you'll need the Microsoft C++ Build Tools:
1. Download [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Install with the "Desktop development with C++" workload

### Optional: Virtual COM (Linux)

For virtual COM port features on Linux:
```bash
sudo apt-get install socat  # Debian/Ubuntu
sudo yum install socat      # RHEL/CentOS
```

## 🚀 Building and Running

1. Clone or navigate to this repository
2. Build and run the application:

```bash
cargo run --release
```

The `--release` flag builds an optimized version for better performance.

## 📖 Usage Guide

### Basic Connection

1. Click **🔄** to refresh the list of available COM ports
2. Select your **COM port** from the dropdown (shows USB VID/PID if applicable)
3. Configure your **connection settings** (baud rate, data bits, etc.)
4. Click **⚡ Connect** to establish the connection
5. The status will change to **● CONNECTED** when successful

### Sending Data

1. Type your message in the **Send** text box at the bottom
2. Press **Enter** or click **📤 Send**
3. The data will be transmitted with \r\n (CRLF) line ending
4. Sent data appears in the terminal with "TX:" prefix

### Viewing Data

- **ASCII Mode**: Standard text display, great for text-based protocols
- **HEX Mode**: Hexadecimal dump with 16 bytes per line
- **Both Mode**: See hex and ASCII side-by-side

### Using DTR/RTS Controls

1. Connect to a port
2. The **📡 SIGNALS** section will appear
3. Check **DTR** to assert Data Terminal Ready
4. Check **RTS** to assert Request To Send
5. Signals are set immediately on the port

### Logging Data

1. Enable **Enable logging** in the **📝 LOGGING** section
2. Optionally change the log file name
3. All data will be logged with timestamps
4. Click **💾 Save Log** to write the log file

### Filtering Data

1. Enable **Enable filter** in the **🔍 FILTER** section  
2. Enter a regex pattern (e.g., `ERROR.*`)
3. Only matching data will be displayed
4. Invalid patterns show an error message

### Protocol Analysis

1. Select a protocol from the **🔬 PROTOCOL** dropdown
2. The analyzer will interpret incoming data
3. Decoded information appears in the terminal
4. Currently supports Modbus RTU and ASCII

### Virtual COM Ports

1. Enable **Enable emulator** in the **🔄 VIRTUAL COM** section
2. Click **Create Pair** to create virtual port pair
3. Two linked COM ports will be created
4. Data sent to one appears on the other

## 📁 Project Structure

```
RustCOM/
├── Cargo.toml          # Project dependencies
├── src/
│   ├── main.rs         # Main application and GUI
│   └── virtual_com.rs  # Virtual COM port emulator
├── .gitignore
├── LICENSE
└── README.md           # This file
```

## 📦 Dependencies

- **eframe** & **egui** - Modern immediate-mode GUI framework
- **serialport** - Cross-platform serial port communication
- **chrono** - Date and time handling for timestamps
- **regex** - Regular expression support for filtering
- **serde** - Serialization framework

## 🔧 Keyboard Shortcuts

- **Enter** - Send data (when in send text box)
- **Ctrl+C** - Copy selected text
- **Ctrl+V** - Paste into text fields

## 🐛 Troubleshooting

### Port Access Denied
- **Windows**: Close any other applications using the port
- **Linux**: Add your user to the `dialout` group: `sudo usermod -a -G dialout $USER`
- **Mac**: Check permissions on `/dev/tty.*` devices

### Port Not Listed
- Click the **🔄 Refresh Ports** button
- Ensure the device is properly connected
- Check device drivers are installed

### Virtual COM Not Working
- **Windows**: Install com0com driver
- **Linux**: Install socat with your package manager
- Run with appropriate permissions

## 🎯 Use Cases

- **Embedded development** - Debug microcontrollers and embedded systems
- **IoT device testing** - Communicate with sensors and actuators
- **Industrial protocols** - Test Modbus devices and other industrial equipment
- **GPS modules** - Parse NMEA data from GPS receivers
- **Modem communication** - Send AT commands to modems
- **Arduino/Raspberry Pi** - Interact with maker boards
- **Serial debugging** - Troubleshoot serial communication issues
- **Protocol development** - Test and develop custom serial protocols

## 🚀 Future Enhancements

- Macro/script support for automated testing
- CTS/DSR signal monitoring
- Custom baud rate input
- Data export in multiple formats (CSV, JSON)
- Capture/replay functionality
- More protocol analyzers (NMEA, AT commands, etc.)
- Terminal emulation modes (VT100, ANSI)
- Multi-port monitoring

## 📄 License

See LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues and pull requests.

## 👏 Acknowledgments

Built with:
- Rust programming language
- egui immediate mode GUI
- serialport-rs library

---

**Note**: RustCOM is designed for professional use but comes with no warranty. Always verify critical communications with appropriate tools.
Rust Based COM Port Analyzer
