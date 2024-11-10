{
  rustPlatform,
  version ? "git",
  rev ? "unknown",
  date ? "19700101",
  lib,

  installShellFiles,
  stdenv,
  darwin,
  rust-jemalloc-sys,

  imagemagick,
}:
let
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../assets
      ../Cargo.toml
      ../Cargo.lock
      (lib.fileset.fromSource (lib.sources.sourceByRegex ../. [ "^yazi-.*" ]))
    ];
  };
in
rustPlatform.buildRustPackage rec {
  pname = "yazi";
  inherit version src;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    #outputHashes = {
    #  "mlua-0.10.0" = "sha256-Xg6/jc+UP8tbJJ6x1sbAgt8ZHt051xEBBcjmikQqYlw=";
    #};
  };

  env = {
    YAZI_GEN_COMPLETIONS = true;
    VERGEN_GIT_SHA = rev;
    VERGEN_BUILD_DATE = builtins.concatStringsSep "-" (builtins.match "(.{4})(.{2})(.{2}).*" date);
  };

  nativeBuildInputs = [
    installShellFiles
    imagemagick
  ];

  buildInputs = [
    rust-jemalloc-sys
  ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [ Foundation ]);

  postInstall = ''
    installShellCompletion --cmd yazi \
      --bash ./yazi-boot/completions/yazi.bash \
      --fish ./yazi-boot/completions/yazi.fish \
      --zsh  ./yazi-boot/completions/_yazi

    # Resize logo
    for RES in 16 24 32 48 64 128 256; do
      mkdir -p $out/share/icons/hicolor/"$RES"x"$RES"/apps
      magick assets/logo.png -resize "$RES"x"$RES" $out/share/icons/hicolor/"$RES"x"$RES"/apps/yazi.png
    done

    mkdir -p $out/share/applications
    install -m644 assets/yazi.desktop $out/share/applications/
  '';

  meta = {
    description = "Blazing fast terminal file manager written in Rust, based on async I/O";
    homepage = "https://github.com/sxyazi/yazi";
    license = lib.licenses.mit;
    mainProgram = "yazi";
  };
}
