#include <stdint.h>

#include "error.h"

extern const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION;
extern const uint24_t TICEVID_BLOCKS_PER_HEADER;
extern const uint24_t TICEVID_HEADER_SIZE;
extern const uint24_t TICEVID_BLOCKS_PER_CHUNK;
extern const uint24_t TICEVID_CHUNK_SIZE;

typedef struct ticevid_video_data {
    char *title;
    uint24_t chunk_first_index;
    uint24_t chunk_length;
    uint8_t *icon;
    uint24_t total_frames;
    uint8_t fps;
    uint8_t video_height;
} ticevid_video_data_t;

typedef struct ticevid_caption_data {
} ticevid_caption_data_t;

typedef struct ticevid_font_data {
} ticevid_font_data_t;

typedef struct ticevid_video_header {
    uint8_t schema_version;
    char *title;
    uint8_t video_table_length;
    ticevid_video_data_t **video_table;
    uint8_t caption_table_length;
    ticevid_caption_data_t **caption_table;
    uint8_t font_table_length;
    ticevid_font_data_t **font_table;
} ticevid_video_header_t;

ticevid_result_t ticevid_video_init(void);

void ticevid_video_cleanup(void);

ticevid_result_t ticevid_video_update(void);
