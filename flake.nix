{
    description = "The Sans Undertale boss fight for the TI-84+ CE";
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
        toolchain = {
            url = "github:myclevorname/flake";
            inputs = {
                nixpkgs.follows = "nixpkgs";
            };
        };
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };
    outputs = {
        self,
        nixpkgs,
        toolchain,
        rust-overlay,
        ...
    }: let
      inherit (nixpkgs) lib;
      systems = [
          "x86_64-linux"
          "x86_64-darwin"
      ];
      pkgsFor = lib.genAttrs systems (system:
          import nixpkgs {
              localSystem.system = system;
              overlays = [(import rust-overlay)];
              config.allowUnfree = true;
          });
    in {
        packages = lib.mapAttrs (system: pkgs: {
            default = toolchain.packages.x86_64-linux.mkDerivation {
                pname = "sans-ti";
                version = "0.0.1";
                src = self;
            };
            # https://gist.github.com/caseyavila/05862db1fcc8b4544bd9dcc9ecc444b9#file-default-nix
            tilp = pkgs.stdenv.mkDerivation {
                name = "tilp";
                src = pkgs.fetchurl {
                    url = "https://www.ticalc.org/pub/unix/tilp.tar.gz";
                    sha256 = "1mww2pjzvlbnjp2z57qf465nilfjmqi451marhc9ikmvzpvk9a3b";
                };
                postUnpack = ''
                	sed -i -e '/AC_PATH_KDE/d' tilp2-1.18/configure.ac || die
                   sed -i \
                       -e 's/@[^@]*\(KDE\|QT\|KIO\)[^@]*@//g' \
                       -e 's/@X_LDFLAGS@//g' \
                       tilp2-1.18/src/Makefile.am || die
                '';
                nativeBuildInputs = with pkgs; [
                    autoreconfHook
                    pkg-config
                    intltool
                    libtifiles2
                    libticalcs2
                    libticables2
                    libticonv
                    gtk2
                ];
                buildInputs = with pkgs; [
                    glib
                ];
            };
        })
        pkgsFor;
        devShells = lib.mapAttrs (system: pkgs: {
            default = pkgs.mkShell {
                inputsFrom = [self.packages.${system}.default];
                packages = with pkgs; [
                    self.packages.${system}.tilp
                    cargo-make
                    (rust-bin.selectLatestNightlyWith (toolchain:
                        toolchain.default.override {
                            extensions = [
                                "rust-analyzer"
                                "rust-src"
                            ];
                        }))
                ];
            };
        })
        pkgsFor;
    };
}
