# Default recipe to show available commands
default:
    @just --list

# Build the project in release mode
build:
    cargo build --release

# Install the binary globally to your Cargo path (~/.cargo/bin)
install: build
    cargo install --path .

# Stop all running mushy instances across all terminals
stop:
    cargo run -- stop --all

# Test: Spawn the default built-in mushy
test-default:
    cargo run

# Test: Spawn a giant, fast-moving mushy
test-giant:
    cargo run -- --size 150 --speed 2.0

# Test: Spawn a tiny, slow-moving mushy
test-tiny:
    cargo run -- --size 20 --speed 0.5

# Test: Spawn a mushy walking clockwise instead of counter-clockwise
test-cw:
    cargo run -- --cw

# Test: Spawn the walking_pollo GIF explicitly
test-pollo:
    cargo run -- --gif ./walking_pollo.gif

# Test: Spawn a randomized army of 5 pets
test-army:
    chmod +x spawn.sh
    ./spawn.sh
