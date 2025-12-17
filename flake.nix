{
  description = "Aetherus Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlaysRustToolchain = final: prev: {
          rustToolchain =
            let
              rust = prev.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default.override {
                extensions = [ "rust-src" "rustfmt" "rust-analyzer" ];
              };
        };

        overlays = [
          rust-overlay.overlays.default
          overlaysRustToolchain
        ];
        pkgs = import nixpkgs {
          inherit system;
          inherit overlays;
        };
    in
    {
      devShells = {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            netcdf
            hdf5
            mpich
            llvmPackages.clang
            cmake # Required for static compile of NetCDF.
          ];

          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

            #LD_LIBRARY_PATH = "${pkgs.hdf5_1_10}/lib:${pkgs.hdf5_1_10.dev}/lib";
          };
        };
      };
    });
}
