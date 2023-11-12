# Commands in here can be run using `just`, see https://just.systems/man/en/ for syntax etc.

# Run all commands using bash by default
set shell := ["bash", "-c"]

# allow positional arguments to commands
set positional-arguments := true

# List available recipes in the order in which they appear in this file
_default:
    @just --list --unsorted

# Run using x11 display protocol on wayland
run-x11-on-wayland:
    WINIT_UNIX_BACKEND=x11 cargo run
