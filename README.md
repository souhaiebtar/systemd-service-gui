# Systemd Service GUI

A modern, cross-platform desktop application built with Rust and Iced for managing systemd services.

## Features

- View all systemd services with their status
- Start, stop, and restart services with a single click
- Real-time service status updates
- Clean, modern GUI interface
- Cross-platform support (Linux, macOS, Windows)

## Prerequisites

- Rust (latest stable)
- systemd (Linux systems only - the application uses systemctl commands)
- For non-Linux systems, the application will show an error as systemd is not available

## Build / Compile

Build a debug binary:

```bash
cargo build
```

Build an optimized release binary:

```bash
cargo build --release
```

Release output:

```bash
./target/release/systemd-service-gui
```

## Generate `.AppImage` (Linux)

This repository includes everything needed to generate a Linux AppImage.

### Prerequisites

- Rust toolchain (`cargo`)
- `curl` or `wget` (used to download `linuxdeploy` and `appimagetool`)
- `patchelf` (required by `linuxdeploy` on most distributions)

Example (Debian/Ubuntu):

```bash
sudo apt install -y patchelf curl
```

### Generate command

```bash
./scripts/build-appimage.sh
```

Generated file:

```bash
dist/appimage/systemd-service-gui-<version>-<arch>.AppImage
```

`linuxdeploy` and `appimagetool` are downloaded automatically into `dist/appimage/tools/`.

Optional flags:

```bash
./scripts/build-appimage.sh --arch x86_64
./scripts/build-appimage.sh --arch aarch64
./scripts/build-appimage.sh --skip-tool-download
```

Run the generated AppImage:

```bash
chmod +x dist/appimage/systemd-service-gui-<version>-<arch>.AppImage
./dist/appimage/systemd-service-gui-<version>-<arch>.AppImage
```

## Running

```bash
cargo run
```

Or run the compiled binary:

```bash
./target/release/systemd-service-gui
```

## Usage

1. **Refresh**: Click the "Refresh" button to reload the list of services
2. **Start**: Click the "Start" button next to a service to start it
3. **Stop**: Click the "Stop" button next to a service to stop it
4. **Restart**: Click the "Restart" button next to a service to restart it

The application displays:
- Service name
- Service description
- Active state (active/inactive)
- Sub-state (running, exited, etc.)

## Requirements

The application requires `systemctl` to be available in the PATH and appropriate permissions to manage services. On most Linux distributions, you'll need to run the application with sudo or as root to control services:

```bash
sudo cargo run
```

Or set up proper polkit rules to allow your user to manage services without root.

## Architecture

The application consists of two main modules:

- `systemd.rs`: Handles all interactions with systemd via the `systemctl` command-line tool
- `main.rs`: The Iced GUI application that displays the service list and handles user interactions

## Dependencies

- `iced = "0.12"`: The GUI framework
- `tokio = "1"`: Async runtime for non-blocking systemctl calls
- `serde` & `serde_json`: For parsing systemctl JSON output

## License

MIT
