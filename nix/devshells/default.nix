{ pkgs, rust-toolchain }:
pkgs.mkShellNoCC {
  name = "tag-rs-dev-shell";

  packages = [
    rust-toolchain
    pkgs.ripgrep
    pkgs.fd
    pkgs.silver-searcher
  ];
}
