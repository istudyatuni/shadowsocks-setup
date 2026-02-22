# mainly inspired by https://shivjm.blog/perfect-docker-images-for-rust-with-nix/
#
# static toolchain inspired by https://github.com/nix-community/fenix/issues/95#issuecomment-1444255098
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs {inherit system;}).pkgsStatic;
        lib = pkgs.lib;

        target = pkgs.stdenv.targetPlatform.rust.rustcTargetSpec;
        toolchain = with fenix.packages.${system};
          combine [
            stable.rustc
            stable.cargo
            targets."${target}".stable.rust-std
          ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };

        meta = fromTOML (builtins.readFile ./Cargo.toml);
        info =
          meta.package
          // {
            bin.name =
              if lib.hasAttr "bin" meta
              then (lib.elemAt meta.bin 0).name
              else meta.package.name;
          };
        rustApp = rustPlatform.buildRustPackage {
          pname = info.name;
          version = info.version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            # git dependencies should be specified here
            outputHashes = {
              "xshell-0.2.7" = "sha256-CX+MM2QxuPJpqYHYdNtF+Y2I5femrhFpXeZKDEDRQYQ=";
            };
          };
          meta.mainProgram = info.bin.name;
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = info.bin.name;
          tag = info.version;
          config = {
            Entrypoint = [(lib.getExe rustApp)];
          };
        };
      in {
        packages = {
          rust = rustApp;
          docker = dockerImage;
        };
        defaultPackage = rustApp;
      }
    );
}
