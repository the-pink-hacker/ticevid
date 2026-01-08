# TICEVid Video Binary Specification

Revision: `0.1.0-WIP`

License: GPL v3.0

Authors:
- ThePinkHacker

A block is 512 bytes.

Types used in this document:

| Type               | Description                                                              |
|--------------------|--------------------------------------------------------------------------|
| `u[8, 16, 24, 32]` | An unsigned little endian number.                                        |
| `i[8, 16, 24, 32]` | A signed little endian number.                                           |
| `bool`             | Should be `0` for true, or `1` for false. Only the first bit is checked. |
| `[T]`              | An array of `T`.                                                         |
| `[T; N]`           | An array of `T` of length `N`.                                           |
| `str`              | Null terminated string of 8-bit text.                                    |
| `&T`               | An offset in bytes to `T` represented as `u24`.                          |
| `?T`               | A nullable offset in bytes to `T` represented as `u24`.                  |

A top level view of the format is you have the header with most of the metadata
and a series of chunks that make up the picture and caption data.

| Name                    | Blocks |
|-------------------------|--------|
| Header                  | 16     |
| Frame 0 Picture Start   | 16     |
| Frame 1 Picture Start   | 16     |
| Frame 2 Picture Start   | 16     |
| ...                     | 16     |
| Frame 100 Picture Start | 16     |
| Frame 0 Picture 0       | 16     |
| Frame 0 Picture 1       | 16     |
| Frame 0 Picture 2       | 16     |
| Frame 1 Picture 0       | 16     |
| Frame 1 Picture 1       | 16     |
| Frame 2 Picture 0       | 16     |
| ...                     | 16     |
| Frame 100 Picture 2     | 16     |
| Caption Chunk 0         | 16     |
| Caption Chunk 1         | 16     |
| Caption Chunk 2         | 16     |
| ...                     | 16     |
| Caption Chunk 100       | 16     |

## Header

All offsets are relative from the start of the start of the header.

| Field                  | Type        | Description                                                 |
|------------------------|-------------|-------------------------------------------------------------|
| `format_version_major` | `u16`       | The format version's major value. Should be `0`.            |
| `format_version_minor` | `u8`        | The format version's minor value. Should be `1`.            |
| `format_version_patch` | `u8`        | The format version's patch value. Should be `0`.            |
| `header_size`          | `u16`       | The size of the header chunk in bytes.[^4]                  |
| `title_count`          | `u8`        | The number of titles.[^4]                                   |
| `title_table`          | `&[&Title]` | All titles that can be played.                              |
| `font_pack`            | `?[u8]`     | The fontlibc fontpack used for captions of menus.[^2]       |
| `ui_font_index`        | `u8`        | The font index into `font_pack` that is used for menus.[^5] |

### Title

| Field                       | Type               | Description                                                                |
|-----------------------------|--------------------|----------------------------------------------------------------------------|
| `name`                      | `?str`             | The name of the title that is displayed to the user.                       |
| `color_palette_count`       | `u8`               | How many colors are provided in the palette.                               |
| `color_palette`             | `?[u16]`           | An array of 1555 colors. Defaults to `xlibc` palette.[^1]                  |
| `icon`                      | `?[u8; 256]`       | A 16x16 icon in the provided palette.                                      |
| `height`                    | `u8`               | The height in pixels of the picture. Should be no higher than `240`.       |
| `frame_count`               | `u24`              | The total number of frames in the video.[^4]                               |
| `fps`                       | `u8`               | The target frames per second the video should run at.                      |
| `caption_track_count`       | `u8`               | The number of caption tracks.                                              |
| `caption_tracks`            | `?[&CaptionTrack]` | All captions in the title.[^1]                                             |
| `caption_foreground`        | `u8`               | The color palette index for the caption text color.                        |
| `caption_background`        | `u8`               | The color palette index for the caption background.                        |
| `caption_transparent`       | `bool`             | Whether the caption background is transparent. Overrides background color. |
| `chapter_count`             | `u8`               | The number of chapters in the title.                                       |
| `chapter_table`             | `?[&Chapter]`      | All chapters in the title.[^1]                                             |
| `picture_chunk_block_count` | `u8`               | The amount of blocks the first picture takes up.[^4]                       |
| `picture_chunk`             | `u24`              | The index of the first picture chunk.                                      |

### Caption Track

| Field               | Type   | Description                                          |
|---------------------|--------|------------------------------------------------------|
| `name`              | `?str` | The name of the caption track displayed to the user. |
| `font_index`        | `u8`   | The index of the font this track uses.               |
| `chunk_block_count` | `u8`   | The amount of blocks the first chunk takes up.[^4]   |
| `chunk_start`       | `u24`  | The index of the first chunk.                        |
| `chunk_count`       | `u24`  | The amount of chunks.[^4]                            |

### Chapter

| Field         | Type   | Description                                    |
|---------------|--------|------------------------------------------------|
| `start_frame` | `u24`  | The frame the chapter starts.                  |
| `name`        | `?str` | The name of the chapter displayed to the user. |

## Chunks

All offsets are relative to the start of the chunk.
Chunks have an alignment of 16 blocks and should never exceed 16 blocks in size.

## Start Picture Chunk

Subsequent picture chunks only contain image data for their payload.

| Field                    | Type           | Description                                                              |
|--------------------------|----------------|--------------------------------------------------------------------------|
| `chunk_start`            | `u24`          | The index of the remaining chunks start at. `0` if last chunk.           |
| `chunk_count`            | `u8`           | The number of remaining chunks in the frame.                             |
| `next_frame_block_count` | `u8`           | The amount of blocks in the next frame's first chunk. `0` if last frame. |
| `picture`                | `PictureChunk` | A modified version of the Quite OK Image format (QOI).                   |

## Picture Chunk

| Field               | Type   | Description                                                                      |
|---------------------|--------|----------------------------------------------------------------------------------|
| `chunk_block_count` | `u8`   | The amount of blocks in the next chunk in the frame. `0` if last chunk in frame. |
| `image_size`        | `u16`  | The length of `image` in bytes.[^4]                                              |
| `image`             | `[u8]` | A modified version of the Quite OK Image format (QOI).                           |

## Caption Chunk

| Field               | Type         | Description                                                      |
|---------------------|--------------|------------------------------------------------------------------|
| `frame`             | `u24`        | The frame index the chunk starts.                                |
| `frame_count`       | `u24`        | The number of frames this chunks lasts.[^4]                      |
| `chunk_block_count` | `u8`         | The amount of blocks the next chunk takes up. `0` if last chunk. |
| `captions`          | `&[Caption]` | The captions in this chunk.                                      |
| `caption_count`     | `u8`         | How many captions are in this chunk.[^4]                         |

### Caption

Caption text should match the selected font's encoding.[^2]

| Field             | Type      | Description                                             |
|-------------------|-----------|---------------------------------------------------------|
| `frame_start`     | `u24`     | Starting frame relative from current frame.             |
| `frame_durration` | `u24`     | How many frames this caption will last.                 |
| `position`        | `u8`      | See [caption position](#caption-position).              |
| `line_count`      | `u8`      | The number of lines.[^4]                                |
| `lines`           | `&[&str]` | Each line of the text to be displayed.                  |

### Caption Position

|        | Left | Center | Right |
|--------|:----:|:------:|:-----:|
| Top    | `0`  | `1`    | `2`   |
| Center | `3`  | `4`    | `5`   |
| Bottom | `6`  | `7`    | `8`   |

[^1]: Only null when count is zero.
[^2]: When no fonts are provided, all strings are expected to be ASCII.
[^3]: Glyphs indices `0` through `127` should follow ASCII.
[^4]: Non-zero value.
[^5]: Should be zero if no font pack is provided.
