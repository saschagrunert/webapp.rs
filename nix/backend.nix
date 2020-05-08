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
  { CARGO_TOML = builtins.readFile ../Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^name = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
, webappVersion ? builtins.readFile (runCommandLocal "project-version"
  { CARGO_TOML = builtins.readFile ../Cargo.toml; } ''
    echo "$CARGO_TOML" | sed -n -e 's/^version = "\(.*\)"$/\1/p' | head -1 | tr -d '\n' > $out
  '')
, webappSrc ? callPackage ./source.nix { inherit webappName webappVersion; }
}:

let
  crateOverrides = pkgs.defaultCrateOverrides // {
    sass-sys = attrs: {
      buildInputs = with pkgs; [ git ];
    };
  };

  buildRustCrate = pkgs.buildRustCrate.override {
    defaultCrateOverrides = crateOverrides;
    inherit rustc cargo;
  };

in (import ../Cargo.nix {
  inherit pkgs lib stdenv;
  inherit buildRustCrate;
  defaultCrateOverrides = crateOverrides;
  rootFeatures = [ "default" ];
}).workspaceMembers.webapp-backend.build.overrideAttrs (attrs: {
  postInstall = ''
    cp -av tls $out
    cp -v ${webappSrc}/Config.toml $out
  '';
})
