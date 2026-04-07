{ pkgs, src, version }:
pkgs.rustPlatform.buildRustPackage {
  pname = "tag-rs";
  inherit version src;
  cargoLock.lockFile = "${src}/Cargo.lock";
}
