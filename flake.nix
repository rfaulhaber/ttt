{
  description = "ttt";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    flake-parts,
    ...
  }: let
    projectName = "ttt";
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [];
      flake.overlays.rustOverlay = inputs.rust-overlay.overlays.default;
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
        "aarch64-linux"
      ];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            self.overlays.rustOverlay
          ];
          config.allowUnfreePredicate = pkg:
            builtins.elem (pkgs.lib.getName pkg) [
              "claude-code"
            ];
        };

        formatter = pkgs.alejandra;

        packages = {
          ${projectName} = pkgs.rustPlatform.buildRustPackage {
            pname = projectName;
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta.mainProgram = projectName;
          };

          default = self'.packages.${projectName};
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            clippy
            rust-analyzer
            cargo-nextest

            claude-code
          ];
        };
      };

      flake = {};
    };
}
