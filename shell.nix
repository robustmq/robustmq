{ pkgs ? import <nixpkgs> { } }:
(pkgs.callPackage
  (
    let
      flake = builtins.getFlake (toString ./.);
    in
    flake.devShells.${builtins.currentSystem}.default
  )
{ })
