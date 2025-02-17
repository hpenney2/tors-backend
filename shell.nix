{ pkgs ? import <nixpkgs> {}
}: pkgs.mkShell {
  nativeBuildInputs = with pkgs.buildPackages; [
    cargo
    rustc
    cmake
  ];
  buildInputs = with pkgs; [
    sqlite
    clang
    libclang
  ];
  
  shellHook = ''
    export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
  '';
}