{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs {}
, stdenvNoCC ? pkgs.stdenvNoCC
, lib ? stdenvNoCC.lib
, runCommandLocal ? pkgs.runCommandLocal
, cleanSource ? lib.cleanSource
, webappName ? builtins.readFile (runCommandLocal "project-name"
  {CARGO_TOML = builtins.readFile ../Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^name = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
, webappVersion ? builtins.readFile (runCommandLocal "project-version"
  {CARGO_TOML = builtins.readFile ../Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^version = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
}:

stdenvNoCC.mkDerivation rec {
  name = "${webappName}-${webappVersion}-source";
  version = webappVersion;
  src = cleanSource ./..;
  preferLocalBuild = true;
  allowSubstitutes = false;
  phases = "installPhase";
  installPhase = ''
    mkdir -p $out
    cp -R $src/. $out;
    chmod +w -R $out
    rm -rf $out/{Cargo.lock,target}
  '';
}
