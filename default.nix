{ nixpkgs ? <nixpkgs>
, pkgs ? (import nixpkgs {})
, stdenv ? pkgs.stdenv
, lib ? stdenv.lib
, runCommandLocal ? pkgs.runCommandLocal
, callPackage ? pkgs.callPackage
, moz_overlay ? (import (builtins.fetchTarball
  "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz"))
, moz_nixpkgs ? (import nixpkgs { overlays = [ moz_overlay ]; })
, date ? null
, channel ? "nightly"
, rustChannel ? moz_nixpkgs.rustChannelOf { inherit date channel; }
, rustc ? rustChannel.rust
, cargo ? rustChannel.cargo
, webappName ? builtins.readFile (runCommandLocal "project-name"
  { CARGO_TOML = builtins.readFile ./Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^name = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
, webappVersion ? builtins.readFile (runCommandLocal "project-version"
  { CARGO_TOML = builtins.readFile ./Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^version = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
, webappSrc ? callPackage ./nix/source.nix { inherit webappName webappVersion; }
} @ args:

let
  frontend = import ./nix/frontend.nix args;
  backend = import ./nix/backend.nix args;

in stdenv.mkDerivation {
  name = "${webappName}-${webappVersion}";
  version = webappVersion;
  src = webappSrc;

  nativeBuildInputs = [ pkgs.makeWrapper ];

  builder = pkgs.writeScript "${webappName}-builder" ''
    source $stdenv/setup

    mkdir $out
    cp -r {${frontend},${backend}}/* $out
    chmod -R +w $out

    wrapProgram $out/bin/backend --run "cd $out"
  '';
}
