# Rusteroids

This is a crappy asteroids mini-game written in Rust, using Bevy engine. The purpose of this project was to learn
Rust in a playful way. Please excuse the awful code; this the first time I'm looking at Rust (and Bevy) and I'm sure
there are many things that could be done better.

> [!IMPORTANT]
> I'm still working on this, so the info below will be incomplete.

## Inspiration

- Beautiful, written in Rust but not using Bevy: https://github.com/justinmimbs/rs-asteroids/tree/master
- In Rust with Bevy but very basic: https://github.com/reu/bevyroids

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
