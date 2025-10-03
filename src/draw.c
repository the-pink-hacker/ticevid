#include <ti/screen.h>
#include <graphx.h>

#include "error.h"
#include "usb.h"
#include "video.h"

ticevid_result_t ticevid_draw_init(void) {
    os_RunIndicOff();
    os_ClrLCDFull();
    gfx_Begin();
    gfx_ZeroScreen();
    gfx_SetDrawBuffer();

    return ticevid_video_init();
}

void ticevid_draw_cleanup(void) {
    gfx_End();
    os_ClrHomeFull();
    os_DrawStatusBar();
    os_HomeUp();
    ticevid_video_cleanup();
}

ticevid_result_t ticevid_draw_update(void) {
    if (ticevid_usb_connected()) {
        ticevid_result_t result = ticevid_video_update();

        if (result != TICEVID_SUCCESS) {
            return result;
        }
    }

    gfx_SwapDraw();

    return TICEVID_SUCCESS;
}
