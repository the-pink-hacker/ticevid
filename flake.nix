{
    description = "A video player for the TI-84 Plus CE";
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
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
            };
        };
    };
    outputs = {
        self,
        nixpkgs,
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
            pkgs = import nixpkgs {
                localSystem.system = system;
                overlays = [(import rust-overlay) tice-rust.overlays.${system}.default];
                config.allowUnfree = true;
            };
        in {
            formatter = pkgs.alejandra;
            devShells = {
                default = pkgs.mkShell {
                    packages = with pkgs; [
                        tilp
                        ti-asset-builder
                        (rust-bin.selectLatestNightlyWith (toolchain:
                            toolchain.default.override {
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
