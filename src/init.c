#include "usb.h"
#include "draw.h"

int ticevid_init(void) {
    ticevid_draw_init();
    ticevid_usb_init();
    return 0;
}

void ticevid_cleanup(void) {
    ticevid_usb_cleanup();
    ticevid_draw_cleanup();
}
