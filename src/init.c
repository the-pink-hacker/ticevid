#include "error.h"
#include "usb.h"
#include "draw.h"

ticevid_result_t ticevid_init(void) {
    EARLY_EXIT(ticevid_draw_init());

    ticevid_usb_init();

    return TICEVID_SUCCESS;
}

void ticevid_cleanup(void) {
    ticevid_usb_cleanup();
    ticevid_draw_cleanup();
}
