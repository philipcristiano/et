{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          package_version = pkgs.lib.removeSuffix "\n" (builtins.readFile ./VERSION);
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          package_name = "et";

          linuxPkgs = import nixpkgs {
            system = "x86_64-linux";
            overlays = overlays;
          };
          package = pkgs.rustPlatform.buildRustPackage {
            pname = package_name;
            version = package_version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ rustToolchain pkgs.tailwindcss ];
          };
          # Build the package targeting linux for the Docker image
          linuxPackage = linuxPkgs.rustPlatform.buildRustPackage {
            pname = package_name;
            version = package_version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ linuxPkgs.rust-bin.stable.latest.default pkgs.tailwindcss ];
          };
        in
        with pkgs;
        {
          devShells.default = mkShell {
            buildInputs = [
                rust-bin.stable.latest.default
                rust-analyzer
                pkgs.postgresql_17
                pkgs.foreman
                pkgs.tailwindcss
                pkgs.opentelemetry-collector
            ];
            shellHook = ''
              export PGDATA=$PWD/pgdata
              export PGDATABASE=et
              export PGUSER=et
            '';
          };
          packages.default = package;
          packages.docker = linuxPkgs.dockerTools.buildLayeredImage {
            name = package_name;
            tag = package_version;
            contents = [ linuxPackage ];
            config = {
              Cmd = [ "/bin/w2z" ];
            };
          };
        }
      );
}
