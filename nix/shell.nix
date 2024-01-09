{ pkgs, ... }:

pkgs.mkShell {
  packages = with pkgs; [
    rustToolchain
    rust-analyzer

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

  buildInputs = with pkgs;
    lib.optionals stdenv.isDarwin
      (with darwin.apple_sdk.frameworks; [ Foundation ]);

  env = { RUST_BACKTRACE = "1"; };
}
