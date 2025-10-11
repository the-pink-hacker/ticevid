#include <stdlib.h>

#include <msddrvce.h>

#include "qoi.h"
#include "usb.h"
#include "video.h"

const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION = 0;
const uint24_t TICEVID_BLOCKS_PER_HEADER = 4;
const uint24_t TICEVID_HEADER_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_HEADER;
const uint24_t TICEVID_BLOCKS_PER_CHUNK = 96;
const uint24_t TICEVID_CHUNK_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_CHUNK;

static uint24_t current_frame = 0;
static uint24_t current_chunk = 0;
static uint24_t last_frame_of_chunk = 0;

ticevid_video_header_t *ticevid_video_header;
ticevid_video_header_t *ticevid_video_chunk;

static bool ticevid_video_is_loaded(void) {
    return ticevid_video_header != NULL;
}

static void offset_pointer(void *pointer, uint24_t offset) {
    if (pointer == NULL) {
        return;
    }

    pointer += offset;
}

ticevid_result_t ticevid_video_load_header(void) {
    ticevid_result_t result = ticevid_usb_copy_chunk(0, TICEVID_BLOCKS_PER_HEADER, (uint8_t *)ticevid_video_header);

    if (result != TICEVID_SUCCESS) {
        return result;
    }

    // Offset all pointers to be global instead of local
    uint24_t offset = (uint24_t)ticevid_video_header;

    offset_pointer(ticevid_video_header->title, offset);
    offset_pointer(ticevid_video_header->video_table, offset);
    offset_pointer(ticevid_video_header->caption_table, offset);
    offset_pointer(ticevid_video_header->font_table, offset);

    uint8_t video_length = ticevid_video_header->video_table_length;
    ticevid_video_data_t **video_table = ticevid_video_header->video_table;

    if (video_length > 0) {
        if (video_table == NULL) {
            return TICEVID_VIDEO_HEADER_INVALID;
        }

        ticevid_video_data_t *video_entries = *video_table;

        for (uint24_t i = 0; i < video_length; i += 3) {
            ticevid_video_data_t *video_entry = video_entries + i;

            if (video_entry == NULL) {
                return TICEVID_VIDEO_HEADER_INVALID;
            }

            offset_pointer(video_entry->title, offset);
            offset_pointer(video_entry->icon, offset);
        }
    }

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_init(void) {
    ticevid_video_header = (ticevid_video_header_t *)malloc(TICEVID_HEADER_SIZE);
    ticevid_video_chunk = (ticevid_video_header_t *)malloc(TICEVID_CHUNK_SIZE);

    if (!ticevid_video_is_loaded() || ticevid_video_chunk == NULL) {
        return TICEVID_VIDEO_CHUNK_MEMORY;
    }

    return TICEVID_SUCCESS;
}

void ticevid_video_cleanup(void) {
    free(ticevid_video_header);
    free(ticevid_video_chunk);
}

ticevid_result_t ticevid_video_play_update(void) {
    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_play_draw(void) {
    return TICEVID_SUCCESS;
}
