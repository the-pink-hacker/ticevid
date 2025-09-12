#include <ti/screen.h>
#include <graphx.h>

#include "error.h"
#include "usb.h"

static uint24_t test = 0;

void ticevid_draw_init(void) {
    os_RunIndicOff();
    os_ClrLCDFull();
    gfx_Begin();
    gfx_ZeroScreen();
    gfx_SetDrawBuffer();
}

void ticevid_draw_cleanup(void) {
    gfx_End();
    os_ClrHomeFull();
    os_DrawStatusBar();
    os_HomeUp();
}

ticevid_result_t ticevid_draw_update(void) {

    if (ticevid_usb_connected()) {
        ticevid_result_t result = ticevid_usb_copy_frame(test);
        test++;
        if (result != TICEVID_SUCCESS) {
            return result;
        }
    }

    gfx_SwapDraw();

    return TICEVID_SUCCESS;
}
