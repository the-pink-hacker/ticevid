#include "error.h"
#include "usb.h"
#include "draw.h"

ticevid_result_t ticevid_init(void) {
    ticevid_result_t result = ticevid_draw_init();

    if (result != TICEVID_SUCCESS) {
        return result;
    }

    ticevid_usb_init();

    return TICEVID_SUCCESS;
}

void ticevid_cleanup(void) {
    ticevid_usb_cleanup();
    ticevid_draw_cleanup();
}
