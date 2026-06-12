# gif_walker

A lightweight, daemonized terminal pet that walks along the inner borders of your terminal using the Kitty Graphics Protocol.

## Requirements
- A terminal emulator with **Kitty Graphics Protocol** support (e.g. WezTerm, Kitty, Ghostty).

## Usage

Start the walker using the default configuration (`config.toml` in the current directory, or XDG config):
```bash
cargo run
```

Start the walker and override the configuration using CLI arguments:
```bash
# Provide a custom GIF
cargo run -- --gif ./my_custom_pet.gif

# Change the target size and rotate clockwise
cargo run -- --size 60 --cw

# Provide a custom config file
cargo run -- --config ./my_config.toml
```

Stop the daemon and clear the screen:
```bash
cargo run -- stop
```

## CLI Arguments

- `-g, --gif <PATH>`: Path to the GIF you want to animate.
- `-s, --size <SIZE>`: The size to scale the GIF bounding box to (in pixels).
- `--cw`: Rotate the GIF clockwise instead of counter-clockwise.
- `-c, --config <PATH>`: Path to a custom `config.toml` file.

## Configuration

The configuration is handled via a TOML file. It will look for a config file in your XDG Config directory (`~/.config/gif_walker/config.toml`) or a custom path passed via `--config`. 

If a config file is not found, it gracefully falls back to default settings and uses the **built-in `mushroom.gif`** that is compiled directly into the binary! This means you can run the executable anywhere without worrying about carrying the original GIF file with you.

Example `config.toml`:

```toml
# Path to the GIF you want to animate (absolute or relative)
# If left out or invalid, falls back to the embedded mushroom!
gif_path = "./mushroom.gif"

# Walk direction. False = Counter-Clockwise, True = Clockwise
rotate_clockwise = false

# The size to scale the GIF bounding box to (in pixels)
target_size = 40
```
