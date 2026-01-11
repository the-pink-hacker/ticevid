#pragma once

#include <stdint.h>

typedef enum [[nodiscard]] {
    TICEVID_SUCCESS = 0,
    TICEVID_USER_EXIT,
    TICEVID_USB_INIT_ERROR,
    TICEVID_USB_ENABLE_ERROR,
    TICEVID_USB_EVENT_ERROR,
    TICEVID_MSD_OPEN_ERROR,
    TICEVID_MSD_READ_ERROR,
    TICEVID_MSD_ASYNC_WAIT,
    TICEVID_VIDEO_HEADER_MEMORY,
    TICEVID_VIDEO_CHUNK_MEMORY,
    TICEVID_VIDEO_CONTAINER_INVALID,
    TICEVID_VIDEO_CONTAINER_NULL,
    TICEVID_VIDEO_CONTAINER_VERSION,
    TICEVID_VIDEO_CONTAINER_TITLE,
    TICEVID_FONT_MISSING,
    TICEVID_FONT_INVALID,
    TICEVID_QOI_TAG,
} ticevid_result_t;

void ticevid_error_print(char *text);

void ticevid_error_set_file_line(char *file, uint24_t line);

// Continues if success, else returns.
#define EARLY_EXIT(a) ({\
    ticevid_result_t result = a;\
    if (result != TICEVID_SUCCESS) {\
        return result;\
    }\
})

// Returns an error and sets debug file and line number
#define RETURN_ERROR(e) ({\
    ticevid_error_set_file_line(__FILE__, __LINE__);\
    return e;\
})
