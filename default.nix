with import <nixpkgs> {};

lib.fix (self: {
  tilp = stdenv.mkDerivation {
     name = "tilp";
     src = fetchurl {
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
     nativeBuildInputs = [ autoreconfHook pkg-config intltool libtifiles2 libticalcs2 libticables2 libticonv gtk2 ];
     buildInputs = [ glib ];
  };
})