{
  description = "Phylax assignment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachSystem [ "aarch64-linux" "aarch64-darwin" "x86_64-linux" ] (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoMeta = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        # Include fibonacci_data.bin in the source package explicitly
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            # Only exclude the flake.lock file, include everything else (like fibonacci_data.bin)
            !(type == "regular" && pkgs.lib.hasSuffix "flake.lock" path);
        };

      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoMeta.package.name;
          version = cargoMeta.package.version;
          src = src;  # Use the modified src with the included file
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = with pkgs; [ gcc pkg-config openssl ];
          nativeBuildInputs = with pkgs; [
            gcc
            pkg-config
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt-preview" "rust-analyzer" ];
            })
          ];
          cargoBuildFlags = [ "" ];

          # Copy the fibonacci_data.bin file to the output directory
          postInstall = ''
            cp ${src}/fibonacci_data.bin $out/fibonacci_data.bin
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            clang
            cargo-flamegraph
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "rustfmt-preview" "rust-analyzer" ];
            })
          ] ++ lib.optional pkgs.stdenv.isLinux [
            pkgs.cargo-llvm-cov
            pkgs.llvm_18
            pkgs.valgrind
            pkgs.gdb
            pkgs.linuxPackages_latest.perf
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          shellHook = ''
            export CARGO_BUILD_RUSTC_WRAPPER=$(which sccache)
            export RUSTC_WRAPPER=$(which sccache)
            export OLD_PS1="$PS1" # Preserve the original PS1
            export PS1="nix-shell:assignment $PS1"

            # For generating code coverage reports using `cargo-llvm-cov`
            export LLVM_COV=/nix/store/smh2gh3sjmj51hrp3vrb6n3lsqda4w3l-llvm-18.1.7/bin/llvm-cov
            export LLVM_PROFDATA=/nix/store/smh2gh3sjmj51hrp3vrb6n3lsqda4w3l-llvm-18.1.7/bin/llvm-profdata
          '';

          # reset ps1
          shellExitHook = ''
            export PS1="$OLD_PS1"
          '';
        };
      }
    );
}
