# gif_walker

A lightweight, daemonized terminal pet that walks along the inner borders of your terminal using the Kitty Graphics Protocol.

## Requirements
- A terminal emulator with **Kitty Graphics Protocol** support (e.g. WezTerm, Kitty, Ghostty).

## Features
- **Multi-Pet Support**: Spawn as many pets as you want in the same terminal! They use completely independent PIDs and uniquely randomized Z-indexes to overlap without graphics tearing.
- **Randomized Spawns**: Pets calculate a pseudo-random perimeter offset when they spawn so they don't start on top of each other.
- **Terminal-Aware Lifecycle**: Pets are intrinsically bound to the specific terminal pane (`tty`) they were launched in.
- **Self-Contained**: The default `mushroom.gif` is compiled directly into the binary! You don't need any external assets to run it.

## Usage

Start the walker using the built-in `mushroom.gif` pet:

```bash
cargo run
```

Start the walker and override the configuration using CLI arguments:
```bash
# Provide a custom GIF
cargo run -- --gif ./walking_pollo.gif

# Change the target size and rotate clockwise
cargo run -- --size 60 --cw

# Change the physical movement speed (does not affect the leg animation rate!)
cargo run -- --speed 2.5

# Provide a custom config file
cargo run -- --config ./my_config.toml
```

Stop the daemon in the **current terminal pane** and clear the screen:
```bash
cargo run -- stop
```

Stop **all** running instances across all terminal windows globally:
```bash
cargo run -- stop --all
```

> **Note:** If you close a terminal window manually, its associated background pets will automatically detect the closure, terminate themselves gracefully, and clean up their graphics from memory!

## CLI Arguments

- `-g, --gif <PATH>`: Path to the GIF you want to animate.
- `-s, --size <SIZE>`: The size to scale the GIF bounding box to (in pixels).
- `-x, --speed <SPEED>`: Multiplier for the physical movement speed along the terminal borders. This affects how fast the pet traverses the screen, independently of the GIF's animation frame rate.
- `--cw`: Rotate the GIF clockwise instead of counter-clockwise.
- `-c, --config <PATH>`: Path to a custom `config.toml` file.

## Configuration

The configuration is handled via a TOML file. It will look for a config file in your XDG Config directory (`~/.config/gif_walker/config.toml`) or a custom path passed via `--config`. 

If a config file is not found, it gracefully falls back to default settings and uses the **built-in `mushroom.gif`**! This means you can run the executable anywhere.

Example `config.toml`:

```toml
# Path to the GIF you want to animate (absolute or relative)
# If left out or invalid, falls back to the embedded mushroom!
gif_path = "./walking_pollo.gif"

# Walk direction. False = Counter-Clockwise, True = Clockwise
rotate_clockwise = false

# The size to scale the GIF bounding box to (in pixels)
target_size = 40

# Speed multiplier (1.0 is normal)
speed = 1.0
```
