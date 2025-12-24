#include <stdint.h>

#include <fontlibc.h>

#include "error.h"

extern const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION;
extern const uint24_t TICEVID_BLOCKS_PER_HEADER;
extern const uint24_t TICEVID_HEADER_SIZE;
extern const uint24_t TICEVID_BLOCKS_PER_CHUNK;
extern const uint24_t TICEVID_CHUNK_SIZE;

typedef struct ticevid_caption_track {
    char *name;
    uint8_t font_index;
    uint16_t chunk_size;
    uint24_t chunk_start;
    uint24_t chunk_count;
} ticevid_caption_track_t;

typedef struct ticevid_chapter {
    uint24_t start_frame;
    char *name;
} ticevid_chapter_t;

typedef struct ticevid_title {
    char *name;
    uint8_t color_palette_count;
    uint16_t *color_palette;
    uint8_t *icon;
    uint8_t height;
    uint24_t frame_count;
    uint8_t fps;
    uint8_t caption_track_count;
    ticevid_caption_track_t **caption_tracks;
    uint8_t caption_foreground;
    uint8_t caption_background;
    bool caption_transparent;
    uint8_t chapter_count;
    ticevid_chapter_t **chapter_table;
    uint16_t picture_chunk_size;
    uint24_t picture_chunk;
} ticevid_title_t;

typedef struct ticevid_container_header {
    uint16_t format_version_major;
    uint8_t format_version_minor;
    uint16_t format_version_patch;
    uint8_t title_count;
    ticevid_title_t **title_table;
    fontlib_font_pack_t *font_pack;
    uint8_t ui_font_index;
} ticevid_container_header_t;

// This buffer is always the max size of the header (TICEVID_HEADER_SIZE)
extern ticevid_container_header_t *ticevid_video_container_header;

ticevid_result_t ticevid_video_init(void);

void ticevid_video_cleanup(void);

ticevid_result_t ticevid_video_load_header(void);

ticevid_result_t ticevid_video_play_update(void);

ticevid_result_t ticevid_video_play_draw(void);
