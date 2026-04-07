{ pkgs, rust-toolchain }:
pkgs.mkShellNoCC {
  name = "tag-rs-dev-shell";

  packages = [
    rust-toolchain
    pkgs.ripgrep
    pkgs.fd
    pkgs.silver-searcher
  ];

  shellHook = ''
    echo ""
    echo "🔧 tag-rs dev shell"
    echo ""
    echo "  build     cargo build"
    echo "  test      cargo test"
    echo "  lint      cargo clippy -- -D warnings && cargo fmt --check"
    echo "  run       cargo run -- <args>"
    echo ""
  '';
}
