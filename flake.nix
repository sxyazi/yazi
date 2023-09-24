{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        yazi = pkgs.callPackage ./nix/yazi.nix { };
      in
      {
        packages.default = yazi;
        packages.yazi = yazi;

        formatter = pkgs.nixpkgs-fmt;

        devShells.default = import ./nix/shell.nix { inherit pkgs inputs; };
      }) 
    // rec {
      overlays = {
        default = overlays.yazi;
        yazi = final: prev: {
          yazi = final.callPackage ./nix/yazi.nix { };
        };
      };
      overlay = overlays.default;
    };
}
