<div align="center">
  <img src="assets/images/logo-512x.png" alt="Tower Defense Logo" width="256" height="256">
</div>

# Tower Defense

I am exploring how much further can I get using AI + my project management and coding skills in game development

## How to play

**Village Defender** is a tower defense game where you protect your village from waves of enemies by building defensive towers and collecting resources.

### Game Mechanics
- **Resource Collection**: Hold `E` near trees and rocks to collect wood and stone resources
- **Tower Building**: Press `B` to enter building mode, then select a tower type and place it within range
- **Tower Types**: 
  - **Bow Tower**: 3 wood, 1 rock - Fast firing, moderate damage
  - **Crossbow Tower**: 10 wood, 3 rock - Slower firing, high damage
- **Enemy Waves**: Defend against increasingly difficult waves of enemies
- **Village Health**: Your village has 200 health - if it reaches 0, you lose!

### Keyboard Shortcuts

#### Movement
- `WASD` or `Arrow Keys` - Move your character around the world

#### Game Controls
- `P` - Start/Resume game (from menu)
- `Space` - Pause/Unpause game
- `B` - Toggle building mode
- `Escape` or `Right Mouse Button` - Cancel building mode or tower selection

#### Resource Collection
- `E` (Hold) - Collect resources from trees and rocks (hold for 1 second)

#### Debug/Development (F-keys)
- `F3` - Toggle chunk HUD display
- `F4`/`F5` - Decrease/Increase unload cap per frame
- `F6`/`F7` - Decrease/Increase load cap per frame  
- `F8`/`F9` - Decrease/Increase active chunk radius
- `F10`/`F11` - Decrease/Increase chunk hysteresis

## How to get started

### 1. Install `rustup`

Follow the official guide: [rustup.rs](https://rustup.rs)

### 2. System requirements (graphics/audio)

#### Windows 10/11:

Install the latest GPU drivers (NVIDIA/AMD/Intel). Vulkan/DX12 runtimes come with the drivers.

#### macOS 11+ (Intel or Apple Silicon)

- Xcode Command Line Tools: `xcode-select --install`
- Metal-capable GPU (built-in on modern Macs)

#### Linux

Vulkan loader, drivers, building tools

##### Arch based

```sh
sudo pacman -Syu --noconfirm vulkan-icd-loader vulkan-tools 
# plus the driver for your GPU (e.g. nvidia-utils or mesa)
```

##### Debian/Ubuntu

```sh
sudo apt update && sudo apt install -y \
  libvulkan1 vulkan-tools mesa-vulkan-drivers \
  pkg-config libudev-dev libasound2-dev libxkbcommon-dev libwayland-dev
```


### Start

```sh
cargo run    # launch the game
# (optional) cargo test
```

That’s it — with rustup and current GPU drivers in place, Bevy/wgpu will pick the best backend automatically (Vulkan/Metal/DirectX) for your platform


## Optional but recommended: mise CLI

It is optional, although I recommend using `mise` to manage project tasks and developer tools. It makes getting started and running common commands consistent across Windows, macOS, and Linux.

- Install `mise` by following the official guide: [mise installation](https://mise.jdx.dev/installing-mise.html)

Project tasks (from the repo root):

```bash
# Install any tool versions declared by the project (if present)
mise install

# Format, check, build, run
mise run fmt
mise run check
mise run build
mise run "build:release"

# Run the game (debug)
mise run dev

# Run the game with dev tools enabled (debug)
mise run "dev:tools"

# Run the game (release)
mise run start

# Clean transient traces produced by the game (project-specific)
mise run clean
```

Notes:
- Using `mise` is optional; you can always run the underlying `cargo` commands directly.
- The commands above map to tasks defined in `.mise.toml` and work on Windows, macOS, and Linux.

## Development tools

- This project supports optional Bevy dev tools (frame time graph and UI debug helpers) behind a Cargo feature.
- Use `mise` tasks to run with or without them:

```bash
# Without dev tools (debug)
mise run dev

# With dev tools enabled (debug)
mise run "dev:tools"

# Release run
mise run start
```

Notes:
- Dev tools are excluded by default from normal runs and builds; they are enabled via `--features devtools`.
- Cross‑platform: works on Windows, Linux, and macOS.

