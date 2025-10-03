#include <graphx.h>
#include <msddrvce.h>

#include "qoi.h"
#include "usb.h"
#include "video.h"

const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION = 0;
const uint24_t TICEVID_BLOCKS_PER_HEADER = 4;
const uint24_t TICEVID_HEADER_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_HEADER;
const uint24_t TICEVID_BLOCKS_PER_CHUNK = 96;
const uint24_t TICEVID_CHUNK_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_CHUNK;

// This buffer is always the max size of the header (TICEVID_HEADER_SIZE)
static ticevid_video_header_t *video_header = NULL;

static bool ticevid_video_is_loaded(void) {
    return video_header != NULL;
}

static ticevid_result_t ticevid_video_load_header(void) {
    ticevid_result_t result = ticevid_usb_copy_chunk(0, TICEVID_BLOCKS_PER_HEADER, (uint8_t *)video_header);

    if (result != TICEVID_SUCCESS) {
        return result;
    }

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_init(void) {
    video_header = (ticevid_video_header_t *)malloc(TICEVID_HEADER_SIZE);

    if (!ticevid_video_is_loaded()) {
        return TICEVID_VIDEO_CHUNK_MEMORY;
    }

    return TICEVID_SUCCESS;
}

void ticevid_video_cleanup(void) {
    free(video_header);
}

ticevid_result_t ticevid_video_update(void) {
    if (!ticevid_video_is_loaded()) {
        ticevid_result_t result = ticevid_video_load_header();

        return result;
    } else {
        gfx_Wait();

        return TICEVID_SUCCESS;
    }
}
