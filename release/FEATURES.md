# RustCOM Feature Guide

## 🎨 UI Improvements (Completed)

### Modern Dark Theme
- Professional dark color scheme
- Reduced eye strain for long sessions
- Consistent styling throughout

### Enhanced Layout
- **Centered buttons** with proper sizing and spacing
- **Grouped settings** with visual separators
- **Emoji icons** for better visual identification
- **Status indicators** with color coding (green/red/gray)
- **Responsive panels** that adapt to window size

### Statistics Dashboard
- Real-time byte counters (sent/received)
- Displayed in the top bar for easy monitoring

## 📺 View Modes (Completed)

### ASCII Mode
- Standard text display
- Perfect for text-based protocols
- Shows printable characters

### Hexadecimal Mode
- Full hex dump format
- 16 bytes per line with offset
- ASCII representation on the right
- Format: `0000  XX XX XX XX ... XX XX  ................`

### Both Mode
- Shows hex dump and ASCII simultaneously
- Separated sections for easy comparison
- Best for debugging binary protocols

### Additional View Options
- **Auto-scroll**: Automatically follows new data
- **Timestamps**: Optional timestamp display for debugging

## 📡 DTR/RTS Signal Control (Completed)

### Features
- **DTR (Data Terminal Ready)**: Toggle to control DTR signal
- **RTS (Request To Send)**: Toggle to control RTS signal
- Changes apply immediately to the serial port
- Visible only when connected
- Useful for hardware flow control and device reset

### Use Cases
- Reset Arduino/ESP32 boards (DTR)
- Hardware handshaking (RTS/CTS)
- Device wake-up signals
- Custom hardware control

## 📝 Data Logging (Completed)

### Capabilities
- **Session logging**: Records all TX/RX data
- **Timestamps**: Precise timestamps for each entry
- **Direction marking**: Clearly labeled TX/RX
- **File export**: Save to custom filename
- **Format**: Human-readable text format

### Default Filename
- Pattern: `rustcom_YYYYMMDD_HHMMSS.log`
- Automatically generated based on session start time

### Log Entry Format
```
[2024-12-09 15:30:45.123] RX: Data received here
[2024-12-09 15:30:47.456] TX: Data sent here
```

### Buttons
- **💾 Save Log**: Write accumulated log entries to file
- **💾 Save**: Save current terminal buffer

## 🔍 Advanced Filtering (Completed)

### Regex-Based Filtering
- Filter incoming data in real-time
- Uses Rust regex syntax
- Only matching data is displayed
- Non-matching data is discarded

### Examples
- `ERROR.*` - Show only lines containing "ERROR"
- `^OK$` - Show only lines that are exactly "OK"
- `\d{3}-\d{4}` - Show lines with pattern like "123-4567"
- `(WARN|ERROR|FATAL)` - Show lines with any of these words

### Error Handling
- Invalid regex patterns show error messages
- Pattern validation happens as you type
- Red error text shows what's wrong

## 🔬 Protocol Analyzer (Completed)

### Supported Protocols

#### Modbus RTU
- Binary protocol used in industrial automation
- Automatic frame detection
- CRC verification
- Function code interpretation

#### Modbus ASCII
- ASCII-based variant of Modbus
- Colon-delimited frames
- LRC checksum verification
- Human-readable format

#### Custom
- Framework for adding your own protocol analyzers
- Extensible architecture

### Future Protocol Support
- NMEA 0183 (GPS)
- AT Commands (Modems)
- PPP/SLIP
- MQTT over serial
- Custom protocols

## 🔄 Virtual COM Emulator (Completed)

### Platform Support

#### Windows
- Requires **com0com** driver
- Creates null-modem COM port pairs
- Download: https://sourceforge.net/projects/com0com/
- Provides helpful error message if not installed

#### Linux
- Uses **socat** utility
- Creates pseudo-terminal pairs
- Install: `sudo apt-get install socat`
- Virtual ports at: `/tmp/vcom0` and `/tmp/vcom1`

#### macOS
- Uses PTY pairs
- Similar to Linux implementation
- Install socat: `brew install socat`

### Use Cases
- **Testing**: Test your serial application without hardware
- **Development**: Develop serial protocols without devices
- **Loopback**: Echo data for verification
- **Simulation**: Simulate device responses

### How It Works
1. Enable "Enable emulator"
2. Click "Create Pair"
3. Two virtual COM ports are created
4. Data sent to one port appears on the other
5. Perfect for testing bidirectional communication

## 🎯 Advanced Features

### Send Options
- **Standard send**: Adds \r\n (CRLF) automatically
- **Send \r\n**: Send just carriage return + line feed
- **Send \n**: Send just line feed
- **Hex input**: Toggle to send hex values (coming soon)

### Buffer Management
- **Clear button**: Empties the receive buffer
- **Auto-limit**: Keeps buffer under 100KB
- **Manual save**: Export buffer to timestamped file

### Port Information
- **USB devices**: Shows VID:PID
- **Port types**: USB, PCI, Bluetooth
- **Auto-detection**: Refreshes on demand

## 🔧 Technical Details

### Performance
- **50ms refresh rate** when connected
- **Async read operations** for responsiveness
- **Buffered I/O** for efficiency
- **Mutex-protected** serial port access

### Memory Management
- **Automatic buffer pruning** at 100KB
- **Efficient byte storage** as Vec<u8>
- **On-demand display conversion**

### Thread Safety
- **Arc<Mutex>** for serial port sharing
- **Borrow checker** ensures safety
- **Lock-free display updates**

## 📊 Statistics Tracking

### Counters
- **Bytes Received**: Total bytes read from port
- **Bytes Sent**: Total bytes written to port
- **Real-time updates**: Updates as data flows

### Future Stats
- Connection duration
- Baud rate efficiency
- Error counts
- Frame counts (protocol-specific)

## 🎨 UI Components

### Panels
1. **Top Bar**: Title, branding, statistics
2. **Left Panel**: All settings and controls (280px)
3. **Central Panel**: Terminal display (remaining space)

### Color Scheme
- **Background**: Dark gray (#1e1e1e)
- **Connected**: Green (#3caa3c)
- **Disconnected**: Gray (#808080)
- **Error**: Red (#ff6464)
- **Buttons**: Context-appropriate colors

### Typography
- **Monospace font**: Terminal display
- **Sans-serif**: UI elements
- **Size variations**: Headers larger, labels smaller

## 🚀 Quick Tips

1. **Port won't open?**
   - Check another app isn't using it
   - Try refreshing the port list
   - Verify permissions (Linux users)

2. **Data looks garbled?**
   - Check baud rate matches device
   - Verify data bits/parity/stop bits
   - Try different view modes

3. **Nothing received?**
   - Check DTR/RTS if device needs them
   - Verify TX/RX aren't swapped
   - Look for filter accidentally enabled

4. **Want to test?**
   - Use virtual COM emulator
   - Connect both ports in two instances
   - Send data between them

5. **Logging everything?**
   - Enable logging before connecting
   - Or enable during session
   - Remember to save log when done

## 📚 Example Workflows

### Testing an Arduino
1. Connect USB, select COM port (e.g., COM3)
2. Set baud to 9600 (or your Serial.begin() rate)
3. Set 8-N-1 (8 data bits, no parity, 1 stop bit)
4. Connect
5. Arduino resets (DTR)
6. See Serial.print() output in terminal
7. Send commands back to Arduino

### Debugging Modbus
1. Select appropriate COM port
2. Set baud rate (often 9600 or 19200)
3. Select "Modbus RTU" protocol
4. Connect
5. View decoded Modbus frames
6. Check function codes and data

### Logging GPS Data
1. Connect to GPS module
2. Set baud rate (often 9600 or 4800)
3. Enable logging
4. Set filename like "gps_log.txt"
5. Connect and let it run
6. Save log when done
7. Parse NMEA sentences offline

---

*All features completed and ready to use!*

