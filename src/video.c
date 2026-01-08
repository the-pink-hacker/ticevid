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

// Not a const to avoid gnu-folding-constant warning
#define TICEVID_BUFFER_COUNT 4

//static uint24_t current_frame = 0;
//static uint24_t current_chunk = 0;
//static uint24_t last_frame_of_chunk = 0;

ticevid_container_header_t *ticevid_video_container_header;
void *ticevid_video_chunk[TICEVID_BUFFER_COUNT];

static uint24_t pointer_offset;
static uint24_t pointer_max_offset;

static bool ticevid_video_is_loaded(void) {
    return ticevid_video_container_header != NULL;
}

static ticevid_result_t _offset_pointer(void *pointer) {
    void **deref = (void **)pointer;

    if (*deref == NULL) {
        return TICEVID_VIDEO_CONTAINER_NULL;
    }

    // Out of bounds check
    if ((uint24_t)deref > pointer_max_offset) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    *deref += pointer_offset;

    return TICEVID_SUCCESS;
}

static inline void offset_pointer_null(void *pointer) {
    (void)_offset_pointer(pointer);
}

static inline ticevid_result_t offset_pointer(void *pointer) {
    return _offset_pointer(pointer);
}

static ticevid_result_t ticevid_video_allocate_chunk_buffers() {
    for (uint8_t i = 0; i < TICEVID_BUFFER_COUNT; i++) {
        void *chunk = malloc(TICEVID_CHUNK_SIZE);

        if (chunk == NULL) {
            return TICEVID_VIDEO_CHUNK_MEMORY;
        }

        ticevid_video_chunk[i] = chunk;
    }

    return TICEVID_SUCCESS;
}

static void ticevid_video_free_chunk_buffers() {
    for (uint8_t i = 0; i < TICEVID_BUFFER_COUNT; i++) {
        free(ticevid_video_chunk[i]);
    }
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_caption_track_init(ticevid_caption_track_t *caption) {
    EARLY_EXIT(offset_pointer(&caption->name));
    
    if (caption->chunk_block_count == 0 || caption->chunk_count == 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    return TICEVID_SUCCESS;
}

// Unsures every offset is a valid pointer
static void ticevid_video_chapter_init(ticevid_chapter_t *chapter) {
    offset_pointer_null(chapter->name);
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_title_init(ticevid_title_t *title) {
    offset_pointer_null(&title->name);

    // Count is zero if null
    if (title->color_palette == NULL && title->color_palette_count != 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    offset_pointer_null(&title->color_palette);
    offset_pointer_null(&title->icon);

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

    offset_pointer_null(&title->caption_tracks);

    for (uint8_t i = 0; i < title->caption_track_count; i++) {
        EARLY_EXIT(offset_pointer(&title->caption_tracks[i]));
        EARLY_EXIT(ticevid_video_caption_track_init(title->caption_tracks[i]));
    }

    // Count is zero if null
    if (title->chapter_table == NULL && title->chapter_count != 0) {
        return TICEVID_VIDEO_CONTAINER_TITLE;
    }

    offset_pointer_null(&title->chapter_table);

    for (uint8_t i = 0; i < title->chapter_count; i++) {
        EARLY_EXIT(offset_pointer(&title->chapter_table[i]));
        ticevid_video_chapter_init(title->chapter_table[i]);
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
static ticevid_result_t ticevid_video_container_init() {
    ticevid_container_header_t *container = ticevid_video_container_header;

    // Ignore patch version
    EARLY_EXIT(check_version(container->version));

    uint24_t header_size = (uint24_t)container->header_size;

    if (header_size == 0 || header_size > TICEVID_HEADER_SIZE) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    // Save some memory by shrinking the buffer to the size
    container = realloc(container, container->header_size);

    if (container == NULL) {
        return TICEVID_VIDEO_HEADER_MEMORY;
    }

    ticevid_video_container_header = container;

    // Offset all pointers to be global instead of local
    pointer_max_offset = (uint24_t)container->header_size;
    pointer_offset = (uint24_t)container;

    if (container->title_count == 0) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    EARLY_EXIT(offset_pointer(&container->title_table));

    for (uint8_t i = 0; i < container->title_count; i++) {
        EARLY_EXIT(offset_pointer(&container->title_table[i]));
        EARLY_EXIT(ticevid_video_title_init(container->title_table[i]));
    }

    // Font index should be zero if no font pack is provided.
    if (container->font_pack == NULL && container->ui_font_index != 0) {
        return TICEVID_VIDEO_CONTAINER_INVALID;
    }

    offset_pointer_null(&container->font_pack);

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_load_header(void) {
    EARLY_EXIT(ticevid_usb_copy_chunk(0, TICEVID_BLOCKS_PER_HEADER, (uint8_t *)ticevid_video_container_header));

    return ticevid_video_container_init();
}

ticevid_result_t ticevid_video_init(void) {
    ticevid_video_container_header = (ticevid_container_header_t *)malloc(TICEVID_HEADER_SIZE);

    // If null
    if (!ticevid_video_is_loaded()) {
        return TICEVID_VIDEO_CHUNK_MEMORY;
    }

    EARLY_EXIT(ticevid_video_allocate_chunk_buffers());

    return TICEVID_SUCCESS;
}

void ticevid_video_cleanup(void) {
    free(ticevid_video_container_header);
    ticevid_video_free_chunk_buffers();
}

ticevid_result_t ticevid_video_play_update(void) {
    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_video_play_draw(void) {
    return TICEVID_SUCCESS;
}
