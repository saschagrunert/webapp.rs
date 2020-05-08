{ nixpkgs ? <nixpkgs>
, pkgs ? (import nixpkgs {})
, stdenv ? pkgs.stdenv
, lib ? stdenv.lib
, runCommandLocal ? pkgs.runCommandLocal
, callPackage ? pkgs.callPackage
, coreutils ? pkgs.coreutils
, rustPlatform ? pkgs.rustPlatform
, git ? pkgs.git
, cargo-web ? pkgs.cargo-web
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
, cargoSha256 ? "0mhc3d19hyhvkzzr0ax3hgy5s1cv8mk6h1h63cml32v84nabcdaj"
, uikit ? callPackage ./uikit {}
, frontendHash ? "62f09f0336d11ebd"
}:

let
  crateOverrides = pkgs.defaultCrateOverrides // {
    sass-sys = attrs: {
      buildInputs = with pkgs; [ git ];
    };
  };

  buildRustCrate = pkgs.buildRustCrate.override {
    defaultCrateOverrides = crateOverrides;
    rustc = (rustc.override {
      targets = [ "wasm32-unknown-unknown" ];
    });
    inherit cargo;
  };

  cargoDeps = rustPlatform.fetchcargo {
    name = webappName;
    version = webappVersion;
    src = webappSrc;
    sha256 = cargoSha256;
    sourceRoot = null;
  };

in (import ../Cargo.nix {
  inherit pkgs lib stdenv;
  inherit buildRustCrate;
  defaultCrateOverrides = crateOverrides;
  rootFeatures = [ "default" ];
}).workspaceMembers.webapp-frontend.build.overrideAttrs (attrs: {
  buildInputs = attrs.buildInputs ++ [
    git cargo-web
  ];

  builder = pkgs.writeScript "frontend-builder.sh" ''
    source $stdenv/setup

    export HOME=$(mktemp -d)

    cp -a ${webappSrc}/. $NIX_BUILD_TOP
    chmod -R +w $NIX_BUILD_TOP

    unpackFile "${cargoDeps}"
    cargoDepsCopy=$(stripHash $(basename ${cargoDeps}))
    chmod -R +w "$cargoDepsCopy"

    mkdir .cargo
    config=${<nixpkgs/pkgs/build-support/rust/fetchcargo-default-config.toml>}
    substitute $config .cargo/config \
      --subst-var-by vendor "$(pwd)/$cargoDepsCopy"

    OUT_DIR=$NIX_BUILD_TOP/target/wasm32-unknown-unknown/release/build/webapp-frontend-${frontendHash}/out/uikit
    mkdir -p $OUT_DIR
    cp -a ${uikit}/. $OUT_DIR
    chmod -R +w $OUT_DIR

    cargo web deploy --release -o $out/static -p webapp-frontend --target wasm32-unknown-unknown

    mkdir -p $lib
  '';
})
