{
  description = "Rust development environment with Clang and LLVM support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        devShell = pkgs.mkShell {
          packages = (with pkgs; [
            # pkg-config 用于管理: 编译、链接时所需库的路径
            pkg-config
            clang_16
            llvm_16
            libxml2
          ]) ++ [
            # Mold Linker for faster builds (only on Linux)
            (pkgs.lib.optionals pkgs.stdenv.isLinux pkgs.mold)
          ] ++ [
            # rust
            pkgs.cargo-insta
            toolchain
          ];
          buildInputs = with pkgs; [ pkgsCross.riscv64.buildPackages.gcc qemu ];
        };
      });
}
