# Systemd Service GUI

A Rust + Iced desktop app to inspect and manage `systemd` services.

## Features

- List all services from `systemctl`
- Start, stop, and restart services
- Filter by service name (live text filter)
- Filter by status buttons:
  - `running`
  - `exited`
  - `dead`
  - `active`
  - `inactive`
- Refresh service list from the UI
- Build and publish Linux AppImage artifacts via GitHub Actions

## Requirements

- Linux with `systemd`
- `systemctl` available in `PATH`
- Permission to manage services (root/sudo or appropriate polkit rules)

## Install (Latest AppImage)

Run this one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/souhaiebtar/systemd-service-gui/main/scripts/install-latest-appimage.sh | bash
```

What the script does:

- Downloads the latest release AppImage from GitHub
- Selects the matching architecture when possible (`x86_64` / `aarch64`)
- Installs it to `~/.local/bin/systemd-service-gui.AppImage`
- Runs `chmod +x` on the AppImage
- Creates desktop entry:
  - `~/.local/share/applications/systemd-service-gui.desktop`
- Creates icon:
  - `~/.local/share/icons/hicolor/scalable/apps/systemd-service-gui.svg`

After install, launch from your app menu or run:

```bash
~/.local/bin/systemd-service-gui.AppImage
```

## Build From Source

### Prerequisites

Install Rust and system dependencies. Example for Debian/Ubuntu:

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  curl \
  libasound2-dev \
  libxkbcommon-dev \
  libwayland-dev \
  libx11-dev \
  libx11-xcb-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxcb-randr0-dev \
  libxi-dev \
  libgl1-mesa-dev \
  libegl1-mesa-dev
```

### Compile

Debug:

```bash
cargo build
```

Release:

```bash
cargo build --release
```

Run release binary:

```bash
./target/release/systemd-service-gui
```

## Build AppImage Locally

The repository includes `scripts/build-appimage.sh`.

Additional prerequisite:

```bash
sudo apt install -y patchelf
```

Build AppImage:

```bash
./scripts/build-appimage.sh
```

Output:

```bash
dist/appimage/systemd-service-gui-<version>-<arch>.AppImage
```

Optional flags:

```bash
./scripts/build-appimage.sh --arch x86_64
./scripts/build-appimage.sh --arch aarch64
./scripts/build-appimage.sh --skip-tool-download
```

## Release Automation

GitHub workflow: `.github/workflows/release-appimage.yml`

- Trigger: push tag matching `v*`
- Also supports manual dispatch
- Builds AppImage and attaches it to the corresponding GitHub Release

Create and push a release tag:

```bash
git tag -a v0.1.4 -m "v0.1.4"
git push origin v0.1.4
```

## Project Structure

- `src/main.rs`: Iced UI and filtering/actions
- `src/systemd.rs`: `systemctl` integration + JSON parsing
- `scripts/build-appimage.sh`: local AppImage builder
- `scripts/install-latest-appimage.sh`: installer for latest release AppImage
- `packaging/appimage/`: desktop file + SVG icon used for AppImage

## License

MIT
