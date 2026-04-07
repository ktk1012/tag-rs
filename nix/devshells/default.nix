{ pkgs, rust-toolchain }:
let
  commands = {
    build = {
      script = "cargo build \"$@\"";
      help = "cargo build";
    };
    test = {
      script = "cargo test \"$@\"";
      help = "cargo test";
    };
    lint = {
      script = "cargo clippy -- -D warnings && cargo fmt --check";
      help = "cargo clippy + fmt check";
    };
    run = {
      script = "cargo run -- \"$@\"";
      help = "cargo run -- <args>";
    };
  };

  commandPackages = builtins.attrValues (
    builtins.mapAttrs (name: cmd: pkgs.writeShellScriptBin name cmd.script) commands
  );

  menu = builtins.concatStringsSep "\n" (
    builtins.attrValues (
      builtins.mapAttrs (name: cmd: "  ${name}\t${cmd.help}") commands
    )
  );
in
pkgs.mkShellNoCC {
  name = "tag-rs-dev-shell";

  packages = [
    rust-toolchain
    pkgs.ripgrep
    pkgs.fd
    pkgs.silver-searcher
  ] ++ commandPackages;

  shellHook = ''
    echo ""
    echo "🔧 tag-rs dev shell"
    echo ""
    echo -e "${menu}"
    echo ""
  '';
}
