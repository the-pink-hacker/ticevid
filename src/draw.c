#include <ti/screen.h>
#include <graphx.h>
#include <fontlibc.h>

#include "error.h"
#include "usb.h"
#include "ui.h"
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
    ticevid_result_t result;

    switch (ui_state) {
        case TICEVID_UI_MAIN:
            gfx_FillScreen(0xFF);
            fontlib_ClearWindow();
            fontlib_HomeUp();
            fontlib_DrawString("TICEVID: The USB Video Player\nPress enter to connect to USB device.");
            break;
        case TICEVID_UI_LOADING_VIDEO_SELECT:
        case TICEVID_UI_LOADING_VIDEO:
            gfx_FillScreen(0xFF);
            fontlib_ClearWindow();
            fontlib_HomeUp();
            fontlib_DrawString("Loading...");
            break;
        case TICEVID_UI_VIDEO_SELECT:
            gfx_FillScreen(0xFF);
            fontlib_ClearWindow();
            fontlib_HomeUp();
            fontlib_DrawString("Select video.");
            fontlib_Newline();
            fontlib_DrawString(ticevid_video_header->title);
            fontlib_Newline();
            fontlib_DrawString("Done.");
            break;
        case TICEVID_UI_PLAYING:
            result = ticevid_video_play_draw();

            if (result != TICEVID_SUCCESS) {
                return result;
            }

            break;
    }

    gfx_SwapDraw();

    return TICEVID_SUCCESS;
}
