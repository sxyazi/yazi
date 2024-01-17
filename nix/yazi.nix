{ makeRustPlatform
, rustToolchain
, version ? "git"
, lib

, makeWrapper
, installShellFiles
, stdenv
, darwin

, withFile ? true
, file
, withJq ? true
, jq
, withPoppler ? true
, poppler_utils
, withUnar ? true
, unar
, withFfmpegthumbnailer ? true
, ffmpegthumbnailer
, withFd ? true
, fd
, withRipgrep ? true
, ripgrep
, withFzf ? true
, fzf
, withZoxide ? true
, zoxide
, imagemagick
}:

(makeRustPlatform { cargo = rustToolchain; rustc = rustToolchain; }).buildRustPackage {
  pname = "yazi";
  inherit version;

  src = ../.;
  YAZI_GEN_COMPLETIONS = true;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ makeWrapper installShellFiles imagemagick ];
  buildInputs = lib.optionals stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [ Foundation ]
  );

  postInstall = with lib;
    let
      runtimePaths = [ ]
        ++ optional withFile file
        ++ optional withJq jq
        ++ optional withPoppler poppler_utils
        ++ optional withUnar unar
        ++ optional withFfmpegthumbnailer ffmpegthumbnailer
        ++ optional withFd fd
        ++ optional withRipgrep ripgrep
        ++ optional withFzf fzf
        ++ optional withZoxide zoxide;
    in
    ''
      wrapProgram $out/bin/yazi \
         --prefix PATH : "${makeBinPath runtimePaths}"
      installShellCompletion --cmd yazi \
        --bash ./yazi-config/completions/yazi.bash \
        --fish ./yazi-config/completions/yazi.fish \
        --zsh  ./yazi-config/completions/_yazi

      # Resize logo
      for RES in 16 24 32 48 64 128 256; do
        mkdir -p $out/share/icons/hicolor/"$RES"x"$RES"/apps
        convert assets/logo.png -resize "$RES"x"$RES" $out/share/icons/hicolor/"$RES"x"$RES"/apps/yazi.png
      done

      mkdir -p $out/share/applications
      install -m644 assets/yazi.desktop $out/share/applications/
    '';

  meta = with lib; {
    description = "Blazing fast terminal file manager written in Rust, based on async I/O";
    homepage = "https://github.com/sxyazi/yazi";
    license = licenses.mit;
  };
}
