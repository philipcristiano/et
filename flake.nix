{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgsFor = nixpkgs.legacyPackages;
          pkgs = import nixpkgs {
            inherit system overlays;
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
            ] ++
              pkgs.lib.optionals pkgs.stdenv.isDarwin [
                darwin.apple_sdk.frameworks.Security # Should only be for darwin
                darwin.apple_sdk.frameworks.SystemConfiguration
            ];
            shellHook = ''
              export PGDATA=$PWD/pgdata
              export PGDATABASE=et
              export PGUSER=et
            '';
          };

        packages.default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix {};
        }
      );
}
