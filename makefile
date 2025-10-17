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

#DEPS = $(BINDIR)/ticevidf.bin
EXTRA_CLEAN = cargo clean

# ----------------------------

include $(shell cedev-config --makefile)

.PHONY: video
video:
	cargo run\
		--bin ticevid-encoder\
		--release\
		--\
		"./resources/video/video.toml"\
		"./bin/video.iso"

$(BINDIR)/ticevidf.bin:
	cargo run\
		--bin asset-builder\
		--release\
		--\
		sprites\
		"./assets/font/ui.toml"\
		"./src/generated/"

