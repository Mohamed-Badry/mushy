# gif_walker

A lightweight, daemonized terminal pet that walks along the inner borders of your terminal using the Kitty Graphics Protocol.

## Requirements
- A terminal emulator with **Kitty Graphics Protocol** support (e.g. WezTerm, Kitty, Ghostty).

## Usage

Start the walker using the default configuration (`config.toml` in the current directory, or XDG config):
```bash
cargo run
```

Start the walker with a custom config file:
```bash
cargo run -- --config ./my_config.toml
```

Stop the daemon and clear the screen:
```bash
cargo run -- stop
```

## Configuration

The configuration is handled via a TOML file. By default, it will look for `--config`, then your XDG Config directory (`~/.config/gif_walker/config.toml`), and if neither exists, it will fall back to default values.

Example `config.toml`:

```toml
# Path to the GIF you want to animate (absolute or relative)
gif_path = "./mushroom.gif"

# Walk direction. False = Counter-Clockwise, True = Clockwise
rotate_clockwise = false

# The size to scale the GIF bounding box to (in pixels)
target_size = 40

# Margins (in terminal cells) to keep the GIF away from the physical window edges.
# Warning: Setting margin_bottom lower than 2 may cause your terminal to autoscroll.
margin_bottom = 2
margin_right = 5
margin_top = 0
margin_left = 0
```
