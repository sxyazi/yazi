{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      inherit (nixpkgs) lib;

      forEachSystem =
        f:
        (lib.listToAttrs (
          map (system: {
            name = system;
            value = f {
              inherit system;
              pkgs = import nixpkgs {
                inherit system;
                overlays = [
                  rust-overlay.overlays.default

                  (
                    final: prev:
                    let
                      toolchain = final.rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; };
                    in
                    {
                      rustPlatform = prev.makeRustPlatform {
                        cargo = toolchain;
                        rustc = toolchain;
                      };
                    }
                  )
                ];
              };
            };
          }) systems
        ));

      rev = self.shortRev or self.dirtyShortRev or "dirty";
      date = self.lastModifiedDate or self.lastModified or "19700101";
      version =
        (builtins.fromTOML (builtins.readFile ./yazi-fm/Cargo.toml)).package.version
        + "pre${builtins.substring 0 8 date}_${rev}";
    in
    {
      packages = forEachSystem (
        { pkgs, system }:
        {
          yazi-unwrapped = pkgs.callPackage ./nix/yazi-unwrapped.nix { inherit version rev date; };
          yazi = pkgs.callPackage ./nix/yazi.nix { inherit (self.packages.${system}) yazi-unwrapped; };
          default = self.packages.${system}.yazi;
        }
      );

      devShells = forEachSystem (
        { pkgs, ... }:
        {
          default = pkgs.callPackage ./nix/shell.nix { };
        }
      );

      formatter = forEachSystem ({ pkgs, ... }: pkgs.nixfmt-rfc-style);

      overlays = {
        default = self.overlays.yazi;
        yazi = _: prev: { inherit (self.packages.${prev.stdenv.system}) yazi yazi-unwrapped; };
      };
    };
}
