let
    pkgs = import <nixpkgs> {};
in pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
        cargo
        cargo-make
        ffmpeg
        fasmg
    ];
}
