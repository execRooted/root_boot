# root_boot

A CLI tool that allows you to restart your PC and then auto-boot into a selected device.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


## Installation

### Automated Installation

### crates.io
```bash
cargo install root_boot
```
**Note: You need Rust 1.70+**

---

### GitHub

1. Clone the repository:
```bash
git clone https://github.com/execRooted/root_boot.git
cd root_boot
```

2. Run the installer:
```bash
sudo ./install.sh
```

The installer will automatically:
- Install Rust if not present
- Install it system-wide to `/usr/local/bin/root_boot`


## Uninstallation

```bash
cd root_boot
sudo ./uninstall.sh
```

## Usage

### Basic Usage

```bash
# Run root_boot (will automatically request sudo if needed)
root_boot

# Or use the short alias
rb
```

The program will:
1. Check for required privileges;
2. Detect all bootable devices on your system;
3. Display them with model names and sizes;
4. Allow you to select a device by number;
5. Set the selected device as the boot device;
6. Reboot the system into that selected device;

### Device Selection

- Devices are displayed as: `Model Size (Path)`
- Enter the number corresponding to your desired boot device
- Enter `0` to exit the program

## Requirements

- **Linux**: Root privileges (automatically requested)
- **Windows**: Administrator privileges (automatically requested)
- **Electricity**: *...well you kinda need that*


---

***Made by execRooted***