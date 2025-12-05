{
    description = "A video player for the TI-84 Plus CE";
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        toolchain = {
            url = "github:the-pink-hacker/ce-toolchain-nix";
            inputs = {
                nixpkgs.follows = "nixpkgs";
            };
        };
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        tice-rust = {
            url = "github:the-pink-hacker/tice-rust";
            inputs = {
                nixpkgs.follows = "nixpkgs";
                rust-overlay.follows = "rust-overlay";
                flake-utils.follows = "flake-utils";
                toolchain.follows = "toolchain";
            };
        };
    };
    outputs = {
        self,
        nixpkgs,
        toolchain,
        rust-overlay,
        tice-rust,
        flake-utils,
        ...
    }:
        flake-utils.lib.eachSystem [
            "x86_64-linux"
            "aarch64-linux"
            "x86_64-darwin"
            "aarch64-darwin"
        ] (system: let
            inherit (nixpkgs) lib;
            pkgs = import nixpkgs {
                inherit system;
                overlays = [
                    (import rust-overlay)
                    tice-rust.overlays.${system}.default
                ];
                config.allowUnfree = true;
            };
        in {
            packages = {
                ticevid = toolchain.packages.${system}.mkDerivation {
                    pname = "ticevid";
                    version = "0.1.0";
                    src = pkgs.nix-gitignore.gitignoreSource [] ./.;
                    nativeBuildInputs = with pkgs; [
                        ti-asset-builder
                    ];
                };
            };
            formatter = pkgs.alejandra;
            devShells = {
                default = pkgs.mkShell {
                    inputsFrom = [self.packages.${system}.ticevid];
                    packages = with pkgs; [
                        tilp
                        (rust-bin.selectLatestNightlyWith (rustToolchain:
                            rustToolchain.default.override {
                                extensions = [
                                    # For debug purposes
                                    "rust-analyzer"
                                    "rust-src"
                                ];
                            }))
                    ];
                };
            };
        });
}
