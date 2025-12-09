# com0com Setup Guide for RustCOM

## What is com0com?

com0com is a Windows kernel-mode virtual serial port driver that creates pairs of virtual COM ports. Data written to one port appears on the paired port, making it perfect for testing serial applications.

## Installation

### Step 1: Download com0com

Download from: https://sourceforge.net/projects/com0com/

Get the latest version (e.g., `com0com-3.0.0.0-i386-and-x64-signed.zip`)

### Step 2: Extract and Install

1. Extract the ZIP file
2. Navigate to the appropriate folder:
   - 64-bit Windows: `x64` folder
   - 32-bit Windows: `i386` folder
3. **Right-click** on `setup.exe` and select **"Run as Administrator"**
4. Follow the installation wizard

### Step 3: Configure Port Pairs

After installation, you have two options:

#### Option A: Using the GUI (Easiest)

1. Open the Start Menu
2. Search for "com0com" or "Setup Command Prompt"
3. Run **"com0com Setup"** as Administrator
4. The GUI will show existing port pairs
5. Port pairs are shown like: `CNCA0 <-> CNCB0`
6. These map to actual COM ports (e.g., COM10 <-> COM11)

#### Option B: Using Command Line

1. Open Command Prompt **as Administrator**
2. Navigate to: `C:\Program Files\com0com\`
3. Run setupc.exe commands:

```cmd
# List current configuration
setupc list

# Install a new port pair (COM98 and COM99)
setupc install PortName=COM98 PortName=COM99

# Or let it auto-assign COM numbers
setupc install

# Remove a port pair (if needed)
setupc remove 0
```

## Verifying Installation

### Check Device Manager

1. Open Device Manager (`devmgmt.msc`)
2. Expand **"com0com - serial port emulator"**
3. You should see pairs like:
   - `com0com - serial port emulator (CNCA0 -> CNCB0)`
   - `com0com - serial port emulator (CNCB0 -> CNCA0)`

### Check COM Port Numbers

1. In Device Manager, expand **"Ports (COM & LPT)"**
2. Look for entries like:
   - `com0com - serial port emulator A0 (COM10)`
   - `com0com - serial port emulator B0 (COM11)`
3. Note the COM numbers - these are what you'll use in RustCOM

## Using with RustCOM

### Method 1: Use Existing Pairs

If you've manually created com0com pairs:

1. In RustCOM, click **"🔄 Refresh Ports"**
2. You should now see your com0com ports in the list (e.g., COM10, COM11)
3. To test:
   - Open RustCOM **twice** (two windows)
   - Window 1: Connect to COM10
   - Window 2: Connect to COM11
   - Type in one window, see it in the other!

### Method 2: Use RustCOM's Virtual COM Feature

1. Click **"🔧 Create/Find Pair"** in the Virtual COM section
2. RustCOM will either:
   - Find existing com0com pairs and display them
   - Try to create new pairs (COM98/COM99)
   - Show helpful instructions if manual setup is needed

3. If successful, you'll see: `✓ COM98 <-> COM99`

## Common Issues

### "Access Denied" when creating pairs

**Solution**: Run RustCOM as Administrator
- Right-click `rustcom.exe` → "Run as Administrator"

### Ports not showing up

**Solution**: 
1. Run `setupc list` to verify pairs exist
2. Check Device Manager
3. Click "Refresh Ports" in RustCOM
4. Restart Windows (sometimes needed after installation)

### Installation fails on Windows 10/11

**Solution**: com0com is unsigned. You may need to:
1. Disable Driver Signature Enforcement temporarily
2. Use the signed version from the download page
3. Or use alternative tools like:
   - Virtual Serial Port Driver (commercial)
   - com2tcp (different approach)

### Can't communicate between ports

**Solution**:
1. Verify both ports are from the same pair (e.g., COM10 and COM11)
2. Check that both ports have matching settings (baud rate, etc.)
   - Note: Virtual ports usually ignore baud rate, but RustCOM still requires it
3. Try toggling DTR/RTS signals

## Example Configuration

### Creating a Test Pair

```cmd
# As Administrator in C:\Program Files\com0com\
setupc install PortName=COM10 PortName=COM11

# Verify
setupc list

# Should show something like:
# CNCA0 PortName=COM10
# CNCB0 PortName=COM11
```

### Testing in RustCOM

1. **Window 1**:
   - Select COM10
   - Baud: 9600 (doesn't matter for virtual ports)
   - Connect
   - Type: "Hello from COM10"

2. **Window 2**:
   - Select COM11
   - Baud: 9600
   - Connect
   - You should see: "Hello from COM10"

## Advanced Options

### Enable/Disable Features

You can modify port behavior with setupc:

```cmd
# Enable/disable null-modem emulation
setupc change CNCA0 EmuBR=yes
setupc change CNCA0 EmuOverrun=yes

# Add port alias
setupc change CNCA0 PortName=COM10,MyPort
```

### Uninstall

To remove com0com:

1. Run `setupc remove *` to remove all pairs
2. Uninstall from Control Panel
3. Or run `setup.exe` and choose uninstall

## Alternatives to com0com

If com0com doesn't work for you:

### Windows
- **Virtual Serial Port Driver** by Eltima (commercial, more stable)
- **com2tcp** (network-based alternative)
- **Free Virtual Serial Ports** by HHD Software

### Linux
- **socat**: `socat -d -d pty,raw,echo=0 pty,raw,echo=0`
- Built into RustCOM's Linux implementation

### macOS
- **socat**: `brew install socat`
- PTY pairs (native)

## Troubleshooting RustCOM Integration

### Error: "No com0com port pairs found"

This means com0com is installed but no pairs are configured.

**Fix**:
```cmd
setupc install PortName=COM98 PortName=COM99
```

Then click "🔧 Create/Find Pair" again in RustCOM.

### Error: "Could not create COM pair with setupc"

This means RustCOM couldn't find or execute setupc.exe.

**Fix**:
1. Manually create pairs using setupc.exe
2. Then just use the ports directly from the port list
3. Don't worry about the Virtual COM feature - the ports already exist!

### Error message shows instructions

The error message in RustCOM will guide you to:
1. Run setupc.exe as Administrator
2. Create a pair manually
3. Check Device Manager

Just follow those instructions, then refresh RustCOM's port list.

## Summary

1. **Install** com0com with administrator rights
2. **Create pairs** using GUI or `setupc install`
3. **Verify** in Device Manager
4. **Use** the COM ports in RustCOM just like real ports
5. **Test** by opening two RustCOM windows

The key insight: Once com0com pairs are created, they appear as regular COM ports in Windows. You don't need RustCOM's "Virtual COM" feature - just select the ports from the normal port list!

---

**Need help?** Check the com0com documentation or the project page at SourceForge.

