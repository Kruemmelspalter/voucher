with import <nixpkgs> { };
mkShell {
  nativeBuildInputs = with pkgs; [
    rustup

    mold
    
    pkg-config

    openssl

    jetbrains.rust-rover
  ];
  PATH = "${builtins.getEnv "PATH"}:${builtins.getEnv "HOME"}/.cargo/bin";
}
