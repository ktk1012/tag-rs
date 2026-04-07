{
  description = "tag-rs: Tag your search results with numbered aliases";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ inputs.rust-overlay.overlays.default ];
            };
          }
        );

      version = "0.1.0";
    in
    {
      packages = forEachSupportedSystem (
        { pkgs, system }:
        {
          tag-rs = pkgs.callPackage ./nix/packages/tag-rs.nix {
            src = self;
            inherit version;
          };

          default = self.packages.${system}.tag-rs;
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs, ... }:
        let
          rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = import ./nix/devshells/default.nix { inherit pkgs rust-toolchain; };
        }
      );
    };
}
