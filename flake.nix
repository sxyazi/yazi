{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        toolchain = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };

        rev = self.shortRev or self.dirtyShortRev or "dirty";
        date = self.lastModifiedDate or self.lastModified or "19700101";
        version =
          (builtins.fromTOML (builtins.readFile ./yazi-fm/Cargo.toml)).package.version
          + "pre${builtins.substring 0 8 date}_${rev}";
      in
      {
        packages = {
          yazi-unwrapped = pkgs.callPackage ./nix/yazi-unwrapped.nix {
            inherit
              version
              rev
              date
              rustPlatform
              ;
          };
          yazi = pkgs.callPackage ./nix/yazi.nix { inherit (self.packages.${system}) yazi-unwrapped; };
          default = self.packages.${system}.yazi;
        };

        devShells = {
          default = pkgs.callPackage ./nix/shell.nix { };
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    )
    // {
      overlays = {
        default = self.overlays.yazi;
        yazi = _: prev: { inherit (self.packages.${prev.stdenv.system}) yazi yazi-unwrapped; };
      };
    };
}
