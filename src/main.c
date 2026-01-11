#include <ti/getkey.h>

#include "error.h"
#include "init.h"
#include "update.h"

static int ticevid_handle_errors(ticevid_result_t result) {
    switch (result) {
        case TICEVID_SUCCESS:
        case TICEVID_USER_EXIT:
            return 0;
        case TICEVID_USB_INIT_ERROR:
            ticevid_error_print("USB init failure.");
            break;
        case TICEVID_USB_ENABLE_ERROR:
            ticevid_error_print("USB enable failure.");
            break;
        case TICEVID_USB_EVENT_ERROR:
            ticevid_error_print("USB event error.");
            break;
        case TICEVID_MSD_OPEN_ERROR:
            ticevid_error_print("USB MSD open failure.");
            break;
        case TICEVID_MSD_READ_ERROR:
            ticevid_error_print("USB MSD read failure.");
            break;
        case TICEVID_VIDEO_HEADER_MEMORY:
            ticevid_error_print("Insufficient memory for video header.");
            break;
        case TICEVID_VIDEO_CHUNK_MEMORY:
            ticevid_error_print("Insufficient memory for video chunk.");
            break;
        case TICEVID_VIDEO_CONTAINER_INVALID:
            ticevid_error_print("Failed to parse container header.");
            break;
        case TICEVID_VIDEO_CONTAINER_NULL:
            ticevid_error_print("Container offset is null.");
            break;
        case TICEVID_VIDEO_CONTAINER_VERSION:
            ticevid_error_print("Container version unsupported.");
            break;
        case TICEVID_VIDEO_CONTAINER_TITLE:
            ticevid_error_print("Failed to parse container title.");
            break;
        case TICEVID_FONT_MISSING:
            ticevid_error_print("Selected font is missing from system.");
            break;
        case TICEVID_FONT_INVALID:
            ticevid_error_print("Selected font is invalid.");
            break;
        case TICEVID_QOI_TAG:
            ticevid_error_print("QOI Tag invalid.");
        // Should never be ran
        case TICEVID_MSD_ASYNC_WAIT:
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
