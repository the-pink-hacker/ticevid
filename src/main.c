#include <ti/getkey.h>

#include "error.h"
#include "init.h"
#include "io.h"
#include "update.h"

static int ticevid_handle_errors(ticevid_result_t result) {
    switch (result) {
        case TICEVID_SUCCESS:
        case TICEVID_USER_EXIT:
            return 0;
        case TICEVID_USB_INIT_ERROR:
            ticevid_io_println("USB init failure.");
            break;
        case TICEVID_USB_ENABLE_ERROR:
            ticevid_io_println("USB enable failure.");
            break;
        case TICEVID_MSD_OPEN_ERROR:
            ticevid_io_println("USB MSD open failure.");
            break;
        case TICEVID_MSD_READ_ERROR:
            ticevid_io_println("USB MSD read failure.");
            break;
        case TICEVID_VIDEO_HEADER_MEMORY:
            ticevid_io_println("Insufficient memory for video header.");
            break;
        case TICEVID_VIDEO_CHUNK_MEMORY:
            ticevid_io_println("Insufficient memory for video chunk.");
            break;
        case TICEVID_VIDEO_CONTAINER_INVALID:
            ticevid_io_println("Failed to parse container header.");
            break;
        case TICEVID_VIDEO_CONTAINER_NULL:
            ticevid_io_println("Container offset is null.");
            break;
        case TICEVID_VIDEO_CONTAINER_VERSION:
            ticevid_io_println("Container version unsupported.");
            break;
        case TICEVID_VIDEO_CONTAINER_TITLE:
            ticevid_io_println("Failed to parse container title.");
            break;
        case TICEVID_FONT_MISSING:
            ticevid_io_println("Selected font is missing from system.");
            break;
        case TICEVID_FONT_INVALID:
            ticevid_io_println("Selected font is invalid.");
            break;
        case TICEVID_QOI_TAG:
            ticevid_io_println("QOI Tag invalid.");
            break;
    }

    while (!os_GetKey());
    return -1;
}

int main(void) {
    ticevid_result_t result = ticevid_init();

    if (result != TICEVID_SUCCESS) {
        ticevid_cleanup();
        return ticevid_handle_errors(result);
    }

    result = ticevid_update_start();

    ticevid_cleanup();

    return ticevid_handle_errors(result);
}
