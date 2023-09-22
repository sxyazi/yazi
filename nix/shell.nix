{ pkgs, inputs, ... }:

pkgs.mkShell {
  packages = with pkgs; [
    nodePackages.cspell

    file
    jq
    poppler_utils
    unar
    ffmpegthumbnailer
    fd
    ripgrep
    fzf
    zoxide
  ];

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin (
    with pkgs.darwin.apple_sdk.frameworks; [ Foundation ]
  );

  inputsFrom = [
    (inputs.devenv.lib.mkShell {
      inherit inputs pkgs;
      modules = [
        ({ pkgs, ... }: {
          languages.rust.enable = true;
        })
      ];
    })
  ];
}
