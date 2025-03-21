{
  description = "rCore dev flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    oldnixpkgs.url = "github:NixOS/nixpkgs/7cf5ccf1cdb2ba5f08f0ac29fc3d04b0b59a07e4";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    oldnixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      oldpkgs = import oldnixpkgs {
        inherit system;
      };
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs;
          [
            (rust-bin.nightly."2024-05-02".minimal.override {
              extensions = [
                "rust-src"
                "llvm-tools"
                "rustfmt"
                "rust-analyzer"
                "rust-docs"
                "clippy"
              ];
              targets = ["riscv64gc-unknown-none-elf"];
            })
            cargo-binutils
            python3
            gdb
            tmux
          ]
          ++ [oldpkgs.qemu];

        # 进入环境后显示rust和qemu版本
        shellHook = ''
          rustc --version
          cargo --version
          qemu-system-riscv64 --version
          qemu-riscv64 --version
        '';
      };
    });
}
