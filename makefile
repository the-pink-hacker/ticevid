# ----------------------------
# Makefile Options
# ----------------------------

NAME = TICEVID
#ICON = icon.png
DESCRIPTION = "USB Video Player"
COMPRESSED = NO

# Give me my booleans
# God ez80 clang is getting out of date
CFLAGS = -Wall -Wextra -Oz -std=c2x
CXXFLAGS = -Wall -Wextra -Oz

# ----------------------------

include $(shell cedev-config --makefile)

.PHONY: video
video:
	cargo run --manifest-path\
		"frame-encoder/Cargo.toml"\
		--release\
		--\
		"./resources/video/video.toml"\
		"./bin/video.iso"
