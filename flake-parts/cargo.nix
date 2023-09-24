{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: let
    # packages required for building the rust packages
    extraPackages = [
      pkgs.pkg-config
    ];
    withExtraPackages = base: base ++ extraPackages;

    craneLib = inputs.crane.lib.${system}.overrideToolchain self'.packages.rust-toolchain;

    commonArgs = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "crates"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "wasm-service-worker";

      nativeBuildInputs = withExtraPackages [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
    };

    deps-only = craneLib.buildDepsOnly ({} // commonArgs);

    packages = let
      buildWasmPackage = {
        name,
        wasm-bindgen-target ? "web",
      }: let
        underscore_name = pkgs.lib.strings.replaceStrings ["-"] ["_"] name;

        wasmArgs =
          commonArgs
          // {
            pname = "${commonArgs.pname}-deps-wasm";
            cargoExtraArgs = "--package ${name}";
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            doCheck = false;
          };

        cargoArtifactsWasm = craneLib.buildDepsOnly (
          wasmArgs
          // {
          }
        );

        cargo-derivation = craneLib.buildPackage ({
            cargoArtifacts = cargoArtifactsWasm;
          }
          // wasmArgs);

        wasm-derivation = pkgs.stdenv.mkDerivation {
          name = "${name}-wasm";
          buildInputs = [pkgs.wasm-bindgen-cli];
          nativeBuildInputs = [pkgs.binaryen];
          src = "";
          buildCommand = ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen \
              ${cargo-derivation}/lib/${underscore_name}.wasm \
              --out-dir $out \
              --target ${wasm-bindgen-target} \

            ${pkgs.binaryen}/bin/wasm-opt \
              -Oz \
              --output $out/${underscore_name}_bg.wasm \
              $out/${underscore_name}_bg.wasm
          '';
        };
      in
        wasm-derivation;
    in {
      loader = buildWasmPackage {
        name = "service-worker-loader";
      };

      worker = buildWasmPackage {
        name = "service-worker";
        # wasm-bindgen-target = "no-modules";
      };

      cargo-doc = craneLib.cargoDoc ({
          cargoArtifacts = deps-only;
        }
        // commonArgs);
    };

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // commonArgs);

      rust-fmt = craneLib.cargoFmt ({
          inherit (commonArgs) src;
        }
        // commonArgs);

      rust-tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // commonArgs);
    };
  in rec {
    inherit packages checks;

    legacyPackages = {
      cargoExtraPackages = extraPackages;
    };
  };
}
