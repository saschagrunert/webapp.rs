{ buildVersion, sha256 }:

{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs {}
, stdenvNoCC ? pkgs.stdenvNoCC
, runCommandLocal ? pkgs.runCommandLocal
, fetchFromGitHub ? pkgs.fetchFromGitHub
, callPackage ? pkgs.callPackage }:

stdenvNoCC.mkDerivation {
  pname = "uikit";
  version = buildVersion;

  src = fetchFromGitHub {
    owner = "uikit";
    repo = "uikit";
    rev = "v${buildVersion}";
    inherit sha256;
  };

  preferLocalBuild = true;
  allowSubstitutes = false;

  phases = "installPhase";
  installPhase = ''
    mkdir -p $out
    cp -R $src/. $out
  '';

  passthru.updateScript = callPackage ./update.nix { inherit buildVersion sha256; };

  meta = with stdenvNoCC.lib; {
    description = "A lightweight and modular front-end framework for developing fast and powerful web interfaces";
    homepage = "https://getuikit.com";
    license = licenses.mit;
    maintainers = with maintainers; [ rick68 ];
    platforms = platforms.all;
  };
}
