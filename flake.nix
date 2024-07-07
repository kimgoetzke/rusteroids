{
  # Thank you, https://github.com/loophp/rust-shell! Most of this is a copy of it.
  description = "Rust-Bevy development shells";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem = {
        config,
        pkgs,
        system,
        ...
      }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };
        makeRustInfo = {
          version,
          profile,
        }: let
          rust = pkgs.rust-bin.${version}.latest.${profile}.override {extensions = ["rust-src"];};
        in {
          name = "rust-" + version + "-" + profile;
          path = "${rust}/lib/rustlib/src/rust/library";
          drvs = [
            pkgs.just
            pkgs.openssl
            pkgs.pkg-config
            pkgs.rust-analyzer
            rust
            pkgs.udev
            pkgs.libudev-zero
            pkgs.alsa-lib
            pkgs.vulkan-loader
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.glibc.dev
            pkgs.libGL
            #pkgs.gcc
            #pkgs.llvmPackages.bintools
            #pkgs.clang
          ];
        };

        makeRustEnv = {
          version,
          profile,
        }: let
          rustInfo = makeRustInfo {
            inherit version profile;
          };
        in
          pkgs.buildEnv {
            name = rustInfo.name;
            paths = rustInfo.drvs;
          };

        matrix = {
          stable-default = {
            version = "stable";
            profile = "default";
          };
          stable-minimal = {
            version = "stable";
            profile = "minimal";
          };
          beta-default = {
            version = "beta";
            profile = "default";
          };
          beta-minimal = {
            version = "beta";
            profile = "minimal";
          };
          nightly-default = {
            version = "nightly";
            profile = "default";
          };
          nightly-minimal = {
            version = "nightly";
            profile = "minimal";
          };
        };
      in {
        formatter = pkgs.alejandra;
        devShells =
          builtins.mapAttrs
          (
            name: value: let
              version = value.version;
              profile = value.profile;
              rustInfo = makeRustInfo {
                inherit version profile;
              };
            in
              pkgs.mkShell {
                name = rustInfo.name;
                RUST_SRC_PATH = rustInfo.path;
                buildInputs = rustInfo.drvs;
              }
          )
          matrix
          // {
            default = let
              version = matrix.stable-default.version;
              profile = matrix.stable-default.profile;
              rustInfo = makeRustInfo {
                inherit version profile;
              };
            in
              pkgs.mkShell {
                name = rustInfo.name;
                RUST_SRC_PATH = rustInfo.path;
                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath rustInfo.drvs;
                #LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_latest.libclang.lib];
                buildInputs = rustInfo.drvs;
                shellHook = ''
                  export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true
                  export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
                  export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
                  echo ""
                  echo "Welcome to your Rust-Bevy development environment!" | ${pkgs.lolcat}/bin/lolcat
                  echo "Start your IDE with: nohup jetbrains-toolbox &"
                  echo ""
                '';
                #BINDGEN_EXTRA_CLANG_ARGS =
                #  # Includes normal include path
                #  (buil#tins.map (a: ''-I"${a}/include"'') [
                #    # Add dev libraries here (e.g. pkgs.libvmi.dev)
                #    pkgs.glibc.dev
                #  ])
                #  # Includes with special directory paths
                #  ++ [
                #    ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
                #    ''-I"${pkgs.glib.dev}/include/glib-2.0"''
                #    ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
                #  ];
              };
          };
      };
    };
}