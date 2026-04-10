#include <stdint.h>

#include <fontlibc.h>

#include "error.h"

extern const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION;
#define TICEVID_BLOCKS_PER_CHUNK 16

typedef struct ticevid_caption_track {
    char *name;
    uint8_t font_index;
    uint8_t chunk_block_count;
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
    uint24_t picture_chunk_table;
} ticevid_title_t;

typedef struct ticevid_container_version {
    uint16_t major;
    uint8_t minor;
    uint8_t patch;
} ticevid_container_version_t;

typedef struct ticevid_container_header {
    ticevid_container_version_t version;
    uint16_t header_size;
    uint8_t title_count;
    ticevid_title_t **title_table;
    fontlib_font_pack_t *font_pack;
    uint8_t ui_font_index;
} ticevid_container_header_t;

typedef struct ticevid_picture_chunk_info {
    uint16_t block_count;
    uint24_t block_index;
} ticevid_picture_chunk_info_t;

#define TICEVID_FRAME_TABLE_BLOCKS sizeof(ticevid_picture_chunk_info_t)

typedef struct ticevid_picture_chunk_table {
    ticevid_picture_chunk_info_t chunks[TICEVID_FRAME_TABLE_BLOCKS];
} ticevid_picture_chunk_table_t;

typedef struct ticevid_picture_chunk {
    uint16_t image_size;
    uint8_t image[];
} ticevid_picture_chunk_t;

extern ticevid_container_header_t *ticevid_video_container_header;

// Unsures every offset is a valid pointer and other validation
ticevid_result_t ticevid_video_container_init(void);

ticevid_result_t ticevid_video_init(void);

void ticevid_video_cleanup(void);

ticevid_result_t ticevid_video_load_header(void);

ticevid_result_t ticevid_video_play_update(void);

ticevid_result_t ticevid_video_play_draw(void);

// Select which title should be played
void ticevid_video_select_title(ticevid_title_t *title);
