# Rusty Asteroids

This is a simple asteroids mini-game written in Rust, using Bevy engine. The purpose of this project is to learn Rust
and Bevy. This is why the code is written in the most horrible Rust you'll ever see. I'm sorry.

## How to run

Ideally, run this project using the provided `direnv` + Nix Flake i.e. after entering the directory, type `direnv allow`
which will automatically create the development environment with all required dependencies for you. Without `direnv`,
you can use the Nix Flake by running `nix develop` in the project directory.

If you're using a JetBrains IDE, you mau also have to update your Run configuration by doing the below:

```shell
# Get the content of LD_LIBRARY_PATH from your shell environment
echo $LD_LIBRARY_PATH

# Add the below PLUS the output of the above as an environment variable
LD_LIBRARY_PATH=$LD_LIBRARY_PATH:
```