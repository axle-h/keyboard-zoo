# Keyboard Zoo

Cross platform, physics based game for children.

* Custom engine, just Rust + SDL2
* Full screen & toddler safe.
* Random dynamic backgrounds.
* Each key press (smash) generates a letter, number or object & plays a sound.
* Objects are realistically simulated in a 2D physics model.
* All graphics & sounds are easily replaceable.

![cover](cover.jpg)

## Build

Requires vcpkg to build.

```bash
cargo install cargo-vcpkg
cargo vcpkg build
cargo build --release
```

All resources are embedded into the binary.

### macOS

The linker will fail to link SDL2 haptics. You will need to add the following to `~/.cargo/config.toml`:

```toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-args=-weak_framework CoreHaptics"]
```

## Config

Config is stored in yaml:

* Windows: `$HOME\AppData\Roaming\rs.keyboard-zoo\`
* macOS: `$HOME/Library/Application\ Support/rs.keyboard-zoo/`
* Linux: `$XDG_CONFIG_HOME/rs.keyboard-zoo` or `$HOME/.config/rs.keyboard-zoo`

Most of it you can ignore except:

* `baby_smash_mode`: enabled -> each key press has an action e.g. A spawns the letter 'A', disabled -> only use mapped controls
* `run_toddler_sandbox`: enable to prevent control keys from working. Requires Administrator on Windows/root on Linux/Accessibility controls on macOS.

### Video Mode

* `Window` (default) - note if your screen is not at least 720p then keyboard-zoo may not even load on first attempt.
    ```yaml
    video:
      mode: !Window
        width: 1280
        height: 720
    ```
* `FullScreen` - native fullscreen (recommended), note keyboard-zoo should scale to any weird resolution but was designed for 1080p & 4k.
    ```yaml
    video:
      mode: !FullScreen
        width: 1920
        height: 1080
    ```




