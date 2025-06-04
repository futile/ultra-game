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

# Build with detailed timing information (doesn't require a full rebuild)
build-with-timings:
    cargo rustc -- -Z time-passes

# Build and output rustc's self-profiling information (doesn't require a full rebuld).
# Allows flamegraph & chrome dev-tools visualization
# See https://github.com/rust-lang/measureme, use `nix shell unstable#measureme` for the binaries.
build-with-rustc-profiling:
    cargo rustc -- -Z self-profile

# Update all cargo-dependencies, including breaking changes
cargo-update-breaking:
    cargo update -Z unstable-options --breaking --verbose && cargo update --verbose
