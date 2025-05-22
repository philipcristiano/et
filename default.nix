{ pkgs ? import <nixpkgs> { } }:
let manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  nativeBuildInputs = [ pkgs.tailwindcss ];
  version = manifest.version;
  cargoLock.lockFile = ./Cargo.lock;
  cargoLock.outputHashes = {
         "axum-embed-0.2.0" = "sha256-hcaL1NR+SNj7j1PIq2a7/2Q3EDrY57jI27yrgD2qef4=";
         "maud-0.26.0" = "sha256-iuO26yBqnNXol0aj701zR9JZWbGEr4pQCpxqzf6gDmY=";
       };
  src = pkgs.lib.cleanSource ./.;

  # Tests require a postgres DB, don't try to do that here
  doCheck = false;

  # build environment variables
  SQLX_OFFLINE = true;
}

