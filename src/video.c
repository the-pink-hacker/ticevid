#include <stdlib.h>

#include <msddrvce.h>
#include <sys/lcd.h>

#include "qoi.h"
#include "usb.h"
#include "video.h"

const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION = 0;
const uint24_t TICEVID_BLOCKS_PER_HEADER = 16;
const uint24_t TICEVID_HEADER_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_HEADER;
const uint24_t TICEVID_BLOCKS_PER_CHUNK = 16;
const uint24_t TICEVID_CHUNK_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_CHUNK;

const uint8_t BUFFER_COUNT = 4;

//static uint24_t current_frame = 0;
//static uint24_t current_chunk = 0;
//static uint24_t last_frame_of_chunk = 0;

ticevid_container_header_t *ticevid_video_container_header;
//ticevid_video_header_t ticevid_video_chunk[BUFFER_COUNT];

static bool ticevid_video_is_loaded(void) {
    return ticevid_video_container_header != NULL;
}

static void offset_pointer_null(void **pointer, uint24_t offset) {
    if (*pointer == NULL) {
        return;
    }

    *pointer += offset;
}

static ticevid_result_t offset_pointer(void **pointer, uint24_t offset) {
    if (*pointer == NULL) {
        return TICEVID_VIDEO_CONTAINER_NULL;
    }

    *pointer += offset;

    return TICEVID_SUCCESS;
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_caption_track_init(ticevid_caption_track_t *caption, uint24_t offset) {
    EARLY_EXIT(offset_pointer(&caption->name, offset));
    
    if (caption->chunk_size == 0 || caption->chunk_count == 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    return TICEVID_SUCCESS;
}

// Unsures every offset is a valid pointer
static void ticevid_video_chapter_init(ticevid_chapter_t *chapter, uint24_t offset) {
    offset_pointer_null(chapter->name, offset);
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_title_init(ticevid_title_t *title, uint24_t offset) {
    offset_pointer_null(&title->name, offset);

    // Count is zero if null
    if (title->color_palette == NULL && title->color_palette_count != 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    offset_pointer_null(&title->color_palette, offset);
    offset_pointer_null(&title->icon, offset);

    if (title->height > LCD_HEIGHT) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    if (title->frame_count == 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    // Count is zero if null
    if (title->caption_tracks == NULL && title->caption_track_count != 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    offset_pointer_null(&title->caption_tracks, offset);

    for (uint8_t i = 0; i < title->caption_track_count; i++) {
        EARLY_EXIT(offset_pointer(&title->caption_tracks[i], offset));
        EARLY_EXIT(ticevid_video_caption_track_init(title->caption_tracks[i], offset));
    }

    // Count is zero if null
    if (title->chapter_table == NULL && title->chapter_count != 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    offset_pointer_null(&title->chapter_table, offset);

    for (uint8_t i = 0; i < title->chapter_count; i++) {
        EARLY_EXIT(offset_pointer(&title->chapter_table[i], offset));
        ticevid_video_chapter_init(title->chapter_table[i], offset);
    }

    return TICEVID_SUCCESS;
}

// Supports versions: [0.1.0, 0.2.0)
static ticevid_result_t check_version(ticevid_container_version_t version) {
    if (version.major == 0 && version.minor == 1) {
        return TICEVID_SUCCESS;
    } else {
        return TICEVID_VIDEO_CONTAINER_VERSION;
    }
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_container_init(ticevid_container_header_t *container) {
    // Ignore patch version
    EARLY_EXIT(check_version(container->version));

    // Offset all pointers to be global instead of local
    uint24_t offset = (uint24_t)container;

    if (container->title_count == 0) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    EARLY_EXIT(offset_pointer(&container->title_table, offset));

    for (uint8_t i = 0; i < container->title_count; i++) {
        EARLY_EXIT(offset_pointer(&container->title_table[i], offset));
        EARLY_EXIT(ticevid_video_title_init(container->title_table[i], offset));
    }

    // Font index should be zero if no font pack is provided.
    if (container->font_pack == NULL && container->ui_font_index != 0) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    offset_pointer_null(&container->font_pack, offset);

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_load_header(void) {
    EARLY_EXIT(ticevid_usb_copy_chunk(0, TICEVID_BLOCKS_PER_HEADER, (uint8_t *)ticevid_video_container_header));

    return ticevid_video_container_init(ticevid_video_container_header);
}

ticevid_result_t ticevid_video_init(void) {
    ticevid_video_container_header = (ticevid_container_header_t *)malloc(TICEVID_HEADER_SIZE);
    //ticevid_video_chunk = (ticevid_video_header_t *)malloc(TICEVID_CHUNK_SIZE);

    if (!ticevid_video_is_loaded() || ticevid_video_container_header == NULL) {
        return TICEVID_VIDEO_CHUNK_MEMORY;
    }

    return TICEVID_SUCCESS;
}

void ticevid_video_cleanup(void) {
    free(ticevid_video_container_header);
    //free(ticevid_video_chunk);
}

ticevid_result_t ticevid_video_play_update(void) {
    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_play_draw(void) {
    return TICEVID_SUCCESS;
}
