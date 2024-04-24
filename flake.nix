{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    let
      # Nixpkgs overlays
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain = final.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        })
      ];
    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs { inherit system overlays; };
          versionSuffix = "pre${
            builtins.substring 0 8
            (self.lastModifiedDate or self.lastModified or "19700101")
          }_${self.shortRev or "dirty"}";
          version = (builtins.fromTOML
            (builtins.readFile ./yazi-fm/Cargo.toml)).package.version
          + versionSuffix;

          yazi-unwrapped = pkgs.callPackage ./nix/yazi-unwrapped.nix { inherit version; };
          yazi = pkgs.callPackage ./nix/yazi.nix { inherit yazi-unwrapped; };
        in
        {
          packages = {
            inherit yazi-unwrapped yazi;
            default = yazi;
          };

          formatter = pkgs.nixpkgs-fmt;

          devShells.default = import ./nix/shell.nix { inherit pkgs inputs; };
        })
    // {
      overlays = rec {
        default = yazi;
        yazi = final: prev: {
          yazi-unwrapped = self.packages."${final.system}".yazi-unwrapped;
          yazi = self.packages."${final.system}".yazi;
        };
      };
    };
}
