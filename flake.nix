# inspired by https://shivjm.blog/perfect-docker-images-for-rust-with-nix/
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        lib = pkgs.lib;
        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        meta = fromTOML (builtins.readFile ./Cargo.toml);
        appRustBuild = rustPlatform.buildRustPackage {
          pname = meta.package.name;
          version = meta.package.version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            # git dependencies should be specified here
            outputHashes = {
              "xshell-0.2.7" = "sha256-CX+MM2QxuPJpqYHYdNtF+Y2I5femrhFpXeZKDEDRQYQ=";
            };
          };
          meta.mainProgram = (lib.elemAt meta.bin 0).name;
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = meta.package.name;
          config = {
            Entrypoint = [(lib.getExe appRustBuild)];
          };
        };
      in {
        packages = {
          rustPackage = appRustBuild;
          docker = dockerImage;
        };
        defaultPackage = dockerImage;
        devShell = pkgs.mkShell {
          buildInputs = [(rustVersion.override {extensions = ["rust-src"];})];
        };
      }
    );
}
