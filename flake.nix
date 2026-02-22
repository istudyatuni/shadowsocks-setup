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
        pkgs = import nixpkgs {inherit system;};
        lib = pkgs.lib;

        target = pkgs.pkgsStatic.stdenv.targetPlatform.rust.rustcTargetSpec;
        toolchain = with fenix.packages.${system};
          combine [
            stable.rustc
            stable.cargo
            targets."${target}".stable.rust-std
          ];
        rustPlatform = pkgs.pkgsStatic.makeRustPlatform {
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
        rustApp = {...} @ args:
          rustPlatform.buildRustPackage ({
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
            }
            // args);

        dockerImage = pkgs.dockerTools.buildImage {
          name = info.bin.name;
          tag = info.version;
          config = {
            Entrypoint = [(lib.getExe (rustApp {}))];
          };
        };
      in {
        packages = {
          rust = rustApp {};
          rustFakeCert = rustApp {
            buildFeatures = ["fake-cert"];
            # when just is added here it's used as build tool, e.g, for "just install"
            nativeBuildInputs = with pkgs; [openssl];
            preBuild = "${lib.getExe pkgs.just} gen-fake-cert";
          };
          docker = dockerImage;
        };
        defaultPackage = rustApp {};
      }
    );
}
