{
  description = "A reproducible development environment for RobustMQ";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self
    , nixpkgs
    , crane
    , flake-utils
    , rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      craneLib = crane.mkLib pkgs;

      rustToolchain = pkgs.rust-bin.stable."1.89.0".default.override {
        extensions = [ "rust-src" "rust-analyzer" "rustfmt" ];
      };

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      inherit (cargoToml.workspace.package) version;

      nativeBuildInputs = with pkgs; [
        pkg-config
        openssl
        protobuf
        clang
        cmake
        rdkafka
        zstd
        gfortran
        snappy
      ];

      devTools = with pkgs; [
        buf
        python3
        cargo-deny
        cargo-nextest
        typos
        (hawkeye.overrideAttrs (_: {
          __intentionallyOverridingVersion = true;
          version = "5.8.1";
        }))
      ];

      commonArgs = {
        pname = "robustmq";
        inherit version;
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
        inherit nativeBuildInputs;
      };


      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      robustmq-pkg = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
      });
    in
    {
      packages.default = robustmq-pkg;

      devShells.default = pkgs.mkShell {
        name = "robustmq-dev-shell";

        inputsFrom = [ cargoArtifacts ];
        inherit nativeBuildInputs;

        packages = [ rustToolchain ] ++ devTools;

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        LD_LIBRARY_PATH =
          with pkgs;
          lib.concatStringsSep ":" [
            (lib.makeLibraryPath [ openssl ])
            "${pkgs.stdenv.cc.cc.lib}/lib"
          ];

        shellHook = ''
          echo "Entered RobustMQ development shell. 🚀"

            if [ ! -d "precommit_venv" ]; then
                echo "Creating Python virtual environment for pre-commit..."
                python3 -m venv precommit_venv
            fi

            if [ -z "$VIRTUAL_ENV" ]; then
                echo "Activating Python venv and installing pre-commit dependencies..."
                source ./precommit_venv/bin/activate
                pip3 install -r ./.requirements-precommit.txt
            fi
        '';
      };
    }
    );
}

