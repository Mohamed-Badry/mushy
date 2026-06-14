#!/bin/bash

# Ensure we're using the latest build
cargo build --release

GIF_WALKER="./target/release/gif_walker"
GIFS=("./mushroom.gif" "./walking_pollo.gif")

echo "Spawning 5 random pets..."

for i in {1..5}; do
    # Pick a random GIF
    GIF=${GIFS[$((RANDOM % 2))]}
    
    # Random size between 40 and 150
    SIZE=$((40 + RANDOM % 111))
    
    # Random speed between 0.5 and 1.5
    # Generate integer between 5 and 15
    R=$((5 + RANDOM % 11))
    SPEED="$((R / 10)).$((R % 10))"

    echo "[$i/5] Spawning $GIF | Size: $SIZE | Speed: $SPEED"
    $GIF_WALKER -g "$GIF" -s "$SIZE" -x "$SPEED"
    
    # Slight delay to ensure deterministic stagger
    sleep 0.1
done
