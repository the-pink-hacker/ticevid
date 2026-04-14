#include <stdlib.h>

#include <msddrvce.h>
#include <sys/lcd.h>
#include <graphx.h>
#include <fontlibc.h>
#include <ti/sprintf.h>

#include "qoi.h"
#include "usb.h"
#include "video.h"
#include "io.h"

const uint8_t TICEVID_DEFAULT_SCHEMA_VERSION = 0;
const uint24_t TICEVID_HEADER_SIZE = MSD_BLOCK_SIZE * TICEVID_BLOCKS_PER_CHUNK;
const uint24_t TICEVID_BUFFER_SIZE = MSD_BLOCK_SIZE * TICEVID_BUFFER_BLOCKS;
// Always aligned to block size
const uint24_t TICEVID_FRAME_TABLE_COUNT = MSD_BLOCK_SIZE;
const uint24_t TICEVID_FRAME_TABLE_SIZE = TICEVID_FRAME_TABLE_COUNT * TICEVID_FRAME_TABLE_BLOCKS;

ticevid_container_header_t *ticevid_video_container_header;
static void *picture_buffer;
static ticevid_picture_chunk_table_t *picture_chunk_table;

static uint24_t current_frame = -1;
static uint24_t picture_chunk_table_index = -1;
static uint24_t picture_chunk_table_block;

static uint24_t pointer_offset;
static uint24_t pointer_max_offset;

static ticevid_title_t *selected_title;
static uint24_t max_pixels = LCD_SIZE;
// How many pixels offset should the image be
static uint24_t pixel_offset;

static bool ticevid_video_is_loaded(void) {
    return ticevid_video_container_header != NULL;
}

static ticevid_result_t _offset_pointer(void *pointer) {
    void **deref = (void **)pointer;

    if (*deref == NULL) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_NULL);
    }

    // Out of bounds check
    if ((uint24_t)*deref > pointer_max_offset) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_INVALID);
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

static ticevid_result_t ticevid_video_allocate_buffers(void) {
    ticevid_video_container_header = (ticevid_container_header_t *)malloc(TICEVID_HEADER_SIZE);

    // If null
    if (!ticevid_video_is_loaded()) {
        RETURN_ERROR(TICEVID_VIDEO_CHUNK_MEMORY);
    }

    picture_buffer = malloc(TICEVID_BUFFER_SIZE);

    if (picture_buffer == NULL) {
        RETURN_ERROR(TICEVID_VIDEO_CHUNK_MEMORY);
    }

    picture_chunk_table = (ticevid_picture_chunk_table_t *)malloc(TICEVID_FRAME_TABLE_SIZE);

    if (picture_chunk_table == NULL) {
        RETURN_ERROR(TICEVID_VIDEO_CHUNK_MEMORY);
    }

    return TICEVID_SUCCESS;
}

static void ticevid_video_free_buffers(void) {
    free(ticevid_video_container_header);
    free(picture_buffer);
    free(picture_chunk_table);
}

// Unsures every offset is a valid pointer
static ticevid_result_t ticevid_video_caption_track_init(ticevid_caption_track_t *caption) {
    EARLY_EXIT(offset_pointer(&caption->name));
    
    if (caption->chunk_block_count == 0 || caption->chunk_count == 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
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
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
    }

offset_pointer_null(&title->color_palette);
    offset_pointer_null(&title->icon);

    if (title->height > LCD_HEIGHT) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
    }

    if (title->frame_count == 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
    }

    // Count is zero if null
    if (title->caption_tracks == NULL && title->caption_track_count != 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
    }

    offset_pointer_null(&title->caption_tracks);

    for (uint8_t i = 0; i < title->caption_track_count; i++) {
        EARLY_EXIT(offset_pointer(&title->caption_tracks[i]));
        EARLY_EXIT(ticevid_video_caption_track_init(title->caption_tracks[i]));
    }

    // Count is zero if null
    if (title->chapter_table == NULL && title->chapter_count != 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_TITLE);
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
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_VERSION);
    }
}

ticevid_result_t ticevid_video_container_init(void) {
    ticevid_container_header_t *container = ticevid_video_container_header;

    // Ignore patch version
    EARLY_EXIT(check_version(container->version));

    uint24_t header_size = (uint24_t)container->header_size;

    if (header_size == 0 || header_size > TICEVID_HEADER_SIZE) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_INVALID);
    }

    // Save some memory by shrinking the buffer to the size
    container = realloc(container, container->header_size);

    if (container == NULL) {
        RETURN_ERROR(TICEVID_VIDEO_HEADER_MEMORY);
    }

    ticevid_video_container_header = container;

    // Offset all pointers to be global instead of local
    pointer_max_offset = (uint24_t)container->header_size;
    pointer_offset = (uint24_t)container;

    if (container->title_count == 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_INVALID);
    }

    EARLY_EXIT(offset_pointer(&container->title_table));

    for (uint8_t i = 0; i < container->title_count; i++) {
        EARLY_EXIT(offset_pointer(&container->title_table[i]));
        EARLY_EXIT(ticevid_video_title_init(container->title_table[i]));
    }

    // Font index should be zero if no font pack is provided.
    if (container->font_pack == NULL && container->ui_font_index != 0) {
        RETURN_ERROR(TICEVID_VIDEO_CONTAINER_INVALID);
    }

    offset_pointer_null(&container->font_pack);

    return TICEVID_SUCCESS;
}

// Loads but doesn't init a chunk
static ticevid_result_t _load_block(uint32_t block_index, uint24_t block_count, void *buffer) {
    return ticevid_usb_copy_chunk(block_index, block_count, buffer);
}

// Loads and inits the video container header
ticevid_result_t ticevid_video_load_header(void) {
    EARLY_EXIT(_load_block(
        0,
        TICEVID_BLOCKS_PER_CHUNK,
        (uint8_t *)ticevid_video_container_header
    ));

    return TICEVID_SUCCESS;
}


ticevid_result_t ticevid_video_init(void) {
    EARLY_EXIT(ticevid_video_allocate_buffers());

    // Non-header chunks are capped at max chunk size
    pointer_max_offset = TICEVID_HEADER_SIZE;

    return TICEVID_SUCCESS;
}

void ticevid_video_cleanup(void) {
    ticevid_video_free_buffers();
}

static ticevid_result_t _load_chunk_table(uint24_t block) {
    return _load_block(block, TICEVID_FRAME_TABLE_BLOCKS, picture_chunk_table);
}

static ticevid_result_t _load_picture_buffer(uint24_t block, uint16_t block_count) {
    return _load_block(block, block_count, picture_buffer);
}

// Sets up the frame for drawing
ticevid_result_t ticevid_video_play_update(void) {
    uint24_t next_frame = current_frame + 1;

    // If end of video
    if (next_frame >= selected_title->frame_count) {
        current_frame = 0;
    } else {
        current_frame = next_frame;
    }

    // If chunk table buffer runs out
    if (picture_chunk_table_index >= TICEVID_FRAME_TABLE_COUNT) {
        // Advance to chunk
        picture_chunk_table_index = 0;
        picture_chunk_table_block += TICEVID_FRAME_TABLE_BLOCKS;

        EARLY_EXIT(_load_chunk_table(picture_chunk_table_block));
        EARLY_EXIT(ticevid_usb_msd_block());
    } else {
        picture_chunk_table_index++;
    }

    return TICEVID_SUCCESS;
}

static void _draw_debug(ticevid_picture_chunk_info_t *chunk_info) {
    gfx_ZeroScreen();
    fontlib_HomeUp();
    fontlib_SetForegroundColor(0xFF);
    fontlib_SetWindowFullScreen();

    char buffer[64];

    boot_sprintf(
        buffer,
        "Frame: %u/%u",
        (unsigned int)current_frame,
        (unsigned int)selected_title->frame_count
    );

    fontlib_DrawString(buffer);
    fontlib_Newline();

    boot_sprintf(
        buffer,
        "Block Count: %u",
        (unsigned int)chunk_info->block_count
    );

    fontlib_DrawString(buffer);
    fontlib_Newline();

    boot_sprintf(
        buffer,
        "Block Index: %u",
        (unsigned int)chunk_info->block_index
    );

    fontlib_DrawString(buffer);
    fontlib_Newline();

    boot_sprintf(
        buffer,
        "Frame Buf: %u/512",
        (unsigned int)picture_chunk_table_index
    );

    fontlib_DrawString(buffer);
    fontlib_Newline();

    boot_sprintf(
        buffer,
        "Frame Buf Block: %u",
        (unsigned int)picture_chunk_table_block
    );

    fontlib_DrawString(buffer);
}

// Buffers, decodes, and draws the frame
ticevid_result_t ticevid_video_play_draw(void) {
    ticevid_picture_chunk_info_t chunk_info = picture_chunk_table->chunks[picture_chunk_table_index];

    // How many blocks need to be loaded of the current frame
    uint16_t remaining_blocks = chunk_info.block_count;
    // The current block
    uint24_t current_block = chunk_info.block_index;

    // How many blocks to read into the buffer
    uint16_t block_count;

    if (remaining_blocks > TICEVID_BUFFER_BLOCKS) {
        block_count = TICEVID_BUFFER_BLOCKS;
        remaining_blocks -= TICEVID_BUFFER_BLOCKS;
    } else {
        block_count = remaining_blocks;
        remaining_blocks = 0;
    }

    // Async read start
    EARLY_EXIT(_load_picture_buffer(current_block, block_count));

    // Do things in mean time
    ticevid_qoi_init_frame(pixel_offset);
    //_draw_debug(&chunk_info);

    // Async read finish
    EARLY_EXIT(ticevid_usb_msd_block());

    uint24_t remaining_bytes = *(uint24_t *)picture_buffer;
    uint24_t remaining_pixels = max_pixels;

    EARLY_EXIT(ticevid_qoi_decode(
        remaining_bytes,
        &remaining_pixels,
        picture_buffer + sizeof(uint24_t)
    ));

    do {
        // Broken
        break;
        if (remaining_blocks > TICEVID_BUFFER_BLOCKS) {
            block_count = TICEVID_BUFFER_BLOCKS;
            remaining_blocks -= TICEVID_BUFFER_BLOCKS;
        } else {
            block_count = remaining_blocks;
            remaining_blocks = 0;
        }

        current_block += TICEVID_BUFFER_BLOCKS;
        remaining_bytes -= TICEVID_BUFFER_SIZE;

        // Async read start
        EARLY_EXIT(_load_picture_buffer(current_block, block_count));

        // Async read finish
        EARLY_EXIT(ticevid_usb_msd_block());

        EARLY_EXIT(ticevid_qoi_decode(
            remaining_bytes,
            &remaining_pixels,
            picture_buffer
        ));
    } while (remaining_bytes > 0);

    return TICEVID_SUCCESS;
}

void ticevid_video_select_title(ticevid_title_t *title) {
    selected_title = title;
    max_pixels = LCD_WIDTH * title->height;
    pixel_offset = ((LCD_WIDTH * LCD_HEIGHT) - (LCD_WIDTH * title->height)) / 2;
    picture_chunk_table_block = title->picture_chunk_table - TICEVID_FRAME_TABLE_BLOCKS;
}
