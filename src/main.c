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
    }

    while (!os_GetKey());
    return -1;
}

int main(void) {
    ticevid_init();

    ticevid_result_t result = ticevid_update_start();

    ticevid_cleanup();

    return ticevid_handle_errors(result);
}
