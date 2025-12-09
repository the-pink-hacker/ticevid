# ----------------------------
# Makefile Options
# ----------------------------

NAME = TICEVID
ICON = icon.png
DESCRIPTION = "USB Video Player"
COMPRESSED = NO

# Give me my booleans
# God ez80 clang is getting out of date
CFLAGS = -Wall -Wextra -Oz -std=c2x
CXXFLAGS = -Wall -Wextra -Oz

DEPS = $(BINDIR)/TICEVIDF.8xv
EXTRA_CLEAN = cargo clean

# ----------------------------

include $(shell cedev-config --makefile)

.PHONY: video
video: $(BINDIR)/video.bin

$(BINDIR)/video.bin: $(BINDIR)/TICEVIDF.bin
	cargo run\
		--bin ticevid-encoder\
		--release\
		--\
		"./resources/video/video.toml"\
		"$(BINDIR)/video.bin"

$(BINDIR)/TICEVIDF.bin:
	ti-asset-builder\
		fontpack\
		-d\
		"./assets/fontpack/main.toml"\
		-t\
		binary\
		-o\
		"$(BINDIR)/TICEVIDF.bin"

$(BINDIR)/TICEVIDF.8xv: $(BINDIR)/TICEVIDF.bin
	convbin -j bin -k 8xv -i $(BINDIR)/TICEVIDF.bin -o $(BINDIR)/TICEVIDF.8xv -n TICEVIDF -r
