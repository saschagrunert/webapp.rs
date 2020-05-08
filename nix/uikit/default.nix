{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs {}
, callPackage ? pkgs.callPackage
}:

(callPackage ./packages.nix {}).uikit
