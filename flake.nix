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
        versionSuffix = "pre${builtins.substring 0 8 (self.lastModifiedDate or self.lastModified or "19700101")}_${self.shortRev or "dirty"}";
        version = (builtins.fromTOML (builtins.readFile ./app/Cargo.toml)).package.version + versionSuffix;
        yazi = pkgs.callPackage ./nix/yazi.nix { inherit version; };
      in
      {
        packages.default = yazi;
        packages.yazi = yazi;

        formatter = pkgs.nixpkgs-fmt;

        devShells.default = import ./nix/shell.nix { inherit pkgs inputs; };
      });
}
