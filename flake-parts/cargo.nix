{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: let
    devTools = [
      # rust tooling
      self'.packages.rust-toolchain
      pkgs.cargo-audit
      pkgs.cargo-udeps
      pkgs.bacon
      pkgs.wasm-bindgen-cli
      # version control
      pkgs.cocogitto
      inputs'.bomper.packages.cli
      # misc
      pkgs.miniserve
    ];

    # packages required for building the rust packages
    extraPackages = [
      pkgs.pkg-config
    ];
    withExtraPackages = base: base ++ extraPackages;

    craneLib = inputs.crane.lib.${system}.overrideToolchain self'.packages.rust-toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "crates"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "wasm-playground";

      nativeBuildInputs = withExtraPackages [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
    };

    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    packages = {
      default = packages.cli;
      cli = craneLib.buildPackage ({
          pname = "cli";
          cargoArtifacts = deps-only;
          cargoExtraArgs = "--bin cli";
          meta.mainProgram = "cli";
        }
        // common-build-args);

      cargo-doc = craneLib.cargoDoc ({
          cargoArtifacts = deps-only;
        }
        // common-build-args);
    };

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // common-build-args);

      rust-fmt = craneLib.cargoFmt ({
          inherit (common-build-args) src;
        }
        // common-build-args);

      rust-tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // common-build-args);
    };
  in rec {
    inherit packages checks;

    devShells.default = pkgs.mkShell rec {
      packages = withExtraPackages devTools;
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;

      shellHook = ''
        ${config.pre-commit.installationScript}
      '';
    };

    apps = {
      cli = {
        type = "app";
        program = pkgs.lib.getBin self'.packages.cli;
      };
      default = apps.cli;
    };
  };
}
