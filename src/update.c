#include "draw.h"
#include "error.h"
#include "io.h"
#include "usb.h"

static ticevid_result_t ticevid_update_loop(void) {
    if (ticevid_io_pressing_exit()) {
        return TICEVID_USER_EXIT;
    }

    if (!ticevid_usb_connected()) {
        ticevid_result_t result = ticevid_usb_attempt_connection();

        if (result != TICEVID_SUCCESS) {
            return result;
        }
    }

    return ticevid_draw_update();
}

ticevid_result_t ticevid_update_start(void) {
    ticevid_result_t result;

    do {
        result = ticevid_update_loop();
    } while (result == TICEVID_SUCCESS);

    return result;
}
