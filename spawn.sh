#!/bin/bash
GIF_WALKER="./target/release/gif_walker"

$GIF_WALKER -g ./walking_pollo.gif -s 150 -x 0.9
$GIF_WALKER -g ./walking_pollo.gif -s 80 -x 1.1
$GIF_WALKER -s 100 
$GIF_WALKER -s 150 -x 1.25

