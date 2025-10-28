## Development quick start

Very simple. Do 1, 2, 3 — now you can test/develop the game on all platforms.

1) Install Rust (rustup)

```bash
# Windows (PowerShell)
winget install Rustlang.Rustup

# macOS (Terminal)
curl https://sh.rustup.rs -sSf | sh
# or: brew install rustup-init && rustup-init

# Linux (Terminal)
curl https://sh.rustup.rs -sSf | sh
```

2) Install system requirements (graphics/audio)

- Windows 10/11: 
  - Install the latest GPU drivers (NVIDIA/AMD/Intel). Vulkan/DX12 runtimes come with the drivers.
- macOS 11+ (Intel or Apple Silicon): 
  - Xcode Command Line Tools: `xcode-select --install`
  - Metal-capable GPU (built-in on modern Macs). Nothing else to install.
- Linux (modern distro):
  - Vulkan loader + drivers and a few common dev libs:
  - Debian/Ubuntu:
    ```bash
    sudo apt update && sudo apt install -y \
      libvulkan1 vulkan-tools mesa-vulkan-drivers \
      pkg-config libudev-dev libasound2-dev libxkbcommon-dev libwayland-dev
    ```
  - Fedora:
    ```bash
    sudo dnf install -y \
      vulkan-loader vulkan-tools mesa-vulkan-drivers \
      pkgconf-pkg-config systemd-devel alsa-lib-devel libxkbcommon-devel wayland-devel
    ```
  - Arch/Manjaro:
    ```bash
    sudo pacman -Syu --noconfirm vulkan-icd-loader vulkan-tools 
    # plus the driver for your GPU (e.g. nvidia-utils or mesa)
    ```

3) Run

```bash
cargo run    # launch the game
# (optional) cargo test
```

That’s it — with rustup and current GPU drivers in place, Bevy/wgpu will pick the best backend automatically (Vulkan/Metal/DirectX) for your platform.


## Development tools

- This project supports optional Bevy dev tools (frame time graph and UI debug helpers) behind a Cargo feature.
- Use `mise` tasks to run with or without them:

```bash
# Without dev tools
mise run run

# With dev tools enabled
mise run run:devtools

# Release profiles
mise run "run:release"
mise run "run:release:devtools"
```

Notes:
- Dev tools are excluded by default from normal runs and builds; they are enabled via `--features devtools`.
- Cross‑platform: works on Windows, Linux, and macOS.

