{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, naersk, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        }).extend(self: super: rec {
          overloadedRust = self.rust-bin.stable.latest.default;
          rustc = overloadedRust;
          cargo = overloadedRust;
          rust-analyzer = overloadedRust;
        });

        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShells.default = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt rust-analyzer ];
        };
      });
}
