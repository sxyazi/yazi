{ lib
, runCommand
, makeWrapper
, yazi-unwrapped

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
}:


let
  inherit (lib) optional makeBinPath;
  runtimePaths = optional withFile file
    ++ optional withJq jq
    ++ optional withPoppler poppler_utils
    ++ optional withUnar unar
    ++ optional withFfmpegthumbnailer ffmpegthumbnailer
    ++ optional withFd fd
    ++ optional withRipgrep ripgrep
    ++ optional withFzf fzf
    ++ optional withZoxide zoxide;
in
runCommand yazi-unwrapped.name
{
  inherit (yazi-unwrapped) pname version meta;

  nativeBuildInputs = [ makeWrapper ];
} ''
  mkdir -p $out/bin
  ln -s ${yazi-unwrapped}/share $out/share
  ln -s ${yazi-unwrapped}/bin/ya $out/bin/ya
  makeWrapper ${yazi-unwrapped}/bin/yazi $out/bin/yazi \
    --prefix PATH : "${makeBinPath runtimePaths}"
''
