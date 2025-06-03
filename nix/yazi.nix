{
  lib,
  formats,
  runCommand,
  makeWrapper,

  runtimeDeps ? (ps: ps),

  # deps
  file,
  yazi-unwrapped,

  # default optional deps
  jq,
  poppler_utils,
  _7zz,
  ffmpeg,
  fd,
  ripgrep,
  resvg,
  fzf,
  zoxide,
  imagemagick,
  chafa,

  settings ? { },
  plugins ? { },
  flavors ? { },
  initLua ? null,
}:
let
  inherit (lib)
    concatStringsSep
    concatMapStringsSep
    optionalString
    makeBinPath
    mapAttrsToList
    ;

  defaultDeps = [
    jq
    poppler_utils
    _7zz
    ffmpeg
    fd
    ripgrep
    resvg
    fzf
    zoxide
    imagemagick
    chafa
  ];
  runtimePaths = [ file ] ++ (runtimeDeps defaultDeps);

  settingsFormat = formats.toml { };

  files = [
    "yazi"
    "theme"
    "keymap"
  ];

  configHome =
    if (settings == { } && initLua == null && plugins == { } && flavors == { }) then
      null
    else
      runCommand "YAZI_CONFIG_HOME" { } ''
        mkdir -p $out
        ${concatMapStringsSep "\n" (
          name:
          optionalString (settings ? ${name} && settings.${name} != { }) ''
            ln -s ${settingsFormat.generate "${name}.toml" settings.${name}} $out/${name}.toml
          ''
        ) files}

        mkdir $out/plugins
        ${optionalString (plugins != { }) ''
          ${concatStringsSep "\n" (
            mapAttrsToList (name: value: "ln -s ${value} $out/plugins/${name}") plugins
          )}
        ''}

        mkdir $out/flavors
        ${optionalString (flavors != { }) ''
          ${concatStringsSep "\n" (
            mapAttrsToList (name: value: "ln -s ${value} $out/flavors/${name}") flavors
          )}
        ''}


        ${optionalString (initLua != null) "ln -s ${initLua} $out/init.lua"}
      '';
in
runCommand yazi-unwrapped.name
  {
    inherit (yazi-unwrapped) pname version meta;

    nativeBuildInputs = [ makeWrapper ];
  }
  ''
    mkdir -p $out/bin
    ln -s ${yazi-unwrapped}/share $out/share
    ln -s ${yazi-unwrapped}/bin/ya $out/bin/ya
    makeWrapper ${yazi-unwrapped}/bin/yazi $out/bin/yazi \
      --prefix PATH : "${makeBinPath runtimePaths}" \
      ${optionalString (configHome != null) "--set YAZI_CONFIG_HOME ${configHome}"}
  ''
