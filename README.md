# Manchatz Desktop Entry Manager

A GTK4-based desktop application manager for Linux systems that allows you to view, edit, create, and manage `.desktop` files - the standard way Linux applications are registered with the desktop environment.

## Overview

Manchatz Desktop Entry Manager provides a user-friendly graphical interface to manage desktop entries for installed applications. It scans your system for `.desktop` files and allows you to:

- Browse all installed applications (system-wide and user-specific)
- View and edit application properties (name, command, icon, description, etc.)
- Create new desktop entries for custom applications
- Delete unwanted desktop entries (with proper permission handling)
- Search through applications by name or description
- Preview application icons in real-time

## Target Platforms

### Supported Distributions

This application is designed to work on any Linux distribution that follows the [freedesktop.org Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/), including but not limited to:

- **Ubuntu** (all versions and flavors: Ubuntu, Kubuntu, Xubuntu, Lubuntu, Ubuntu MATE, Ubuntu Budgie)
- **Debian** and Debian-based distributions
- **Linux Mint**
- **Pop!_OS**
- **elementary OS**
- **Fedora**
- **openSUSE**
- **Arch Linux** and derivatives (Manjaro, EndeavourOS, etc.)
- **Any other Linux distribution** following freedesktop.org standards

### Supported Desktop Environments

The application works with all major desktop environments, including:

- **GNOME** (including Ubuntu's customized version)
- **KDE Plasma**
- **XFCE**
- **MATE**
- **Cinnamon**
- **Budgie**
- **LXQt / LXDE**
- **Pantheon** (elementary OS)
- Any other desktop environment following freedesktop.org standards

### Package Manager Integration

Manchatz Desktop Entry Manager automatically detects applications installed through various package managers:

- **APT** (`.deb` packages) - Native Debian/Ubuntu packages
- **Snap** - Scans `/var/lib/snapd/desktop/applications`
- **Flatpak** - Scans `/var/lib/flatpak/exports/share/applications`
- **AppImage** - Can create/manage `.desktop` entries for AppImages
- **Manually installed** - User-specific entries in `~/.local/share/applications`

## Features

- **Comprehensive Scanning**: Automatically detects desktop files from multiple locations:
  - `/usr/share/applications` - System-wide applications
  - `/usr/local/share/applications` - Locally installed applications
  - `~/.local/share/applications` - User-specific applications
  - `/var/lib/snapd/desktop/applications` - Snap packages
  - `/var/lib/flatpak/exports/share/applications` - Flatpak packages

- **Permission-Aware Editing**: The application automatically detects whether you have write permissions for each desktop file and disables editing for system files that require elevated privileges.

- **Real-Time Icon Preview**: See application icons as you edit them, supporting both icon names (from icon themes) and direct file paths.

- **Search Functionality**: Quickly find applications by searching through names and descriptions.

- **File Dialogs**: Browse for executables and icon files using native file picker dialogs.

- **Validation**: Ensures desktop entries follow the freedesktop.org specification.

## Requirements

- **GTK4** - Modern GTK toolkit (version 4.x)
- **Rust** - Version 1.70 or newer (for building from source)
- **Linux** - Any distribution with a freedesktop.org-compliant desktop environment

## Installation

### Building from Source

1. **Install Rust** (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install GTK4 development libraries**:

On **Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install libgtk-4-dev build-essential
```

On **Fedora**:
```bash
sudo dnf install gtk4-devel gcc
```

On **Arch Linux**:
```bash
sudo pacman -S gtk4 base-devel
```

3. **Clone and build**:
```bash
git clone https://github.com/yourusername/manchatz-desktop-entry-manager.git
cd manchatz-desktop-entry-manager
cargo build --release
```

4. **Run the application**:
```bash
cargo run --release
```

Or install the binary:
```bash
sudo cp target/release/manchatz-desktop-entry-manager /usr/local/bin/
```

## Usage

### Viewing Applications

Launch the application and all detected desktop entries will be displayed in the left panel. Click on any application to view its details in the right panel.

### Editing Applications

1. Select an application from the list
2. Modify the fields as needed:
   - **Name**: The display name of the application
   - **Command**: The executable command to run
   - **Icon**: Icon name or path to an icon file
   - **Comment**: A brief description of the application
   - **Categories**: Semicolon-separated categories (e.g., `Utility;Development;`)
   - **Run in terminal**: Check if the application should run in a terminal
3. Click "Save Changes" to apply your modifications

**Note**: System-wide applications (in `/usr/share/applications`) are read-only unless you run the application with elevated privileges.

### Creating New Applications

1. Click the "+ New Entry" button
2. Fill in the application details
3. Click "Save Changes" to create the `.desktop` file in `~/.local/share/applications/`

This is particularly useful for:
- Adding custom scripts or programs
- Creating launchers for AppImages
- Adding shortcuts to web applications

### Deleting Applications

1. Select an application
2. Click "Delete Entry"
3. The desktop file will be permanently removed (only works for user-owned files)

### Editing System Applications

To edit system-wide applications that require elevated privileges, you can run:

```bash
sudo -E manchatz-desktop-entry-manager
```

**Warning**: Be cautious when editing system files. Incorrect modifications may prevent applications from launching properly.

## Technical Details

### Desktop File Locations

The application scans the following directories in order:
1. `/usr/share/applications/` - Distribution and package manager installed apps
2. `/usr/local/share/applications/` - Locally compiled/installed apps
3. `~/.local/share/applications/` - User-specific applications
4. `/var/lib/snapd/desktop/applications/` - Snap packages
5. `/var/lib/flatpak/exports/share/applications/` - Flatpak packages

### Desktop Entry Specification

Desktop files follow the [freedesktop.org Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/). Each `.desktop` file is an INI-style configuration file with a `[Desktop Entry]` section containing key-value pairs.

### Permission Handling

The application checks file permissions before allowing edits:
- Files in the user's home directory (`~/.local/share/applications/`) can be edited freely
- System files may require root privileges to modify
- Read-only files show a lock icon and have editing disabled

## Project Structure

- `src/main.rs` - Application entry point and GTK4 initialization
- `src/desktop_file.rs` - Desktop file parser and data model
- `src/ui.rs` - GTK4 user interface implementation

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, or suggest new features.

## License

MIT

## Acknowledgments

- Built with [GTK4](https://gtk.org/) and [gtk4-rs](https://gtk-rs.org/)
- Follows the [freedesktop.org Desktop Entry Specification](https://specifications.freedesktop.org/)

## Troubleshooting

### Application doesn't launch
Make sure you have GTK4 installed:
```bash
# Ubuntu/Debian
sudo apt install libgtk-4-1

# Fedora
sudo dnf install gtk4

# Arch
sudo pacman -S gtk4
```

### Can't see Snap/Flatpak applications
Ensure the respective package managers are installed and the applications directories exist:
```bash
ls /var/lib/snapd/desktop/applications/  # For Snap
ls /var/lib/flatpak/exports/share/applications/  # For Flatpak
```

### Can't edit system applications
System applications require elevated privileges. Run with sudo:
```bash
sudo -E manchatz-desktop-entry-manager
```

## Future Enhancements

Potential features for future releases:
- Bulk editing and management
- Import/export desktop entries
- Validation warnings for invalid desktop files
- Support for additional desktop entry types (links, directories)
- Undo/redo functionality
- Application icon theme browser
- Categories management with predefined options
