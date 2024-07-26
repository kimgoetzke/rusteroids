# Rusteroids

This is a crappy asteroids-like game written in Rust, using Bevy engine and Rapier2d. The purpose of this project was to
learn Rust in a playful way. Please excuse the awful code; this the first time I'm looking at Rust (and Bevy), so there
will be millions of things that could have been done in a better and more idiomatic way.

> [!IMPORTANT]
> I'm still working on this, so the info below will be incomplete.

## Demo

![Demo GIF](./assets/demo.gif)

## Features

- Infinite wave system with increasing difficulty
- Single-button menu to exit the game
- Audio
- Collision system powered by `bevy_rapier2d`
- Particles powered by `bevy_enoki`

## How to develop

### Using Nix Flakes, JetBrains RustRover & Direnv

You can run this project in any way you like, but I have set things up to make it easy to develop using JetBrains
RustRover. For this, you'll need:

- `direnv`
- Any Direnv integration plugin e.g. https://plugins.jetbrains.com/plugin/15285-direnv-integration
- `nix`

This way, you'll just need to `direnv allow` in the project directory after which all prerequisites (incl. Rust, Cargo,
all Bevy dependencies, etc.) will be available to you. The JetBrains plugin will ensure that the environment is
available to your IDE and you can run the project from there (vs `cargo build` and `cargo run` in the terminal).

### Using Nix Flakes

Without `direnv`, you can use the Nix Flake by running `nix develop` in the project directory. If you want to use an IDE
such as JetBrains RustRover, you'll have to set up the environment manually. You'll most likely have to make
`LD_LIBRARY_PATH` available to your IDE.
