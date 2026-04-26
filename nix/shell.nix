{
  mkShell,
  yazi,
  toolchain,
  cspell,
  yazi-unwrapped,
}:

mkShell {
  packages = yazi.passthru.runtimePaths ++ [
    (toolchain.override {
      extensions = [
        "rust-src"
        "rustfmt"
        "rust-analyzer"
        "clippy"
      ];
    })
    cspell
  ];

  inputsFrom = [ yazi-unwrapped ];

  env.RUST_BACKTRACE = "1";
}
