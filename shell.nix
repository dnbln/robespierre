{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.nixpkgs-fmt
    pkgs.niv

    pkgs.clang_12
    pkgs.lld_12
    pkgs.rustup
  ];

  shellHook = ''
    export CC=clang
  '';
}
