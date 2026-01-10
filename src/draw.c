#include <ti/screen.h>
#include <graphx.h>
#include <fontlibc.h>
#include <lcddrvce.h>

#include "error.h"
#include "usb.h"
#include "ui.h"
#include "video.h"

uint8_t *ticevid_vbuffer;

ticevid_result_t ticevid_draw_init(void) {
    lcd_Init();
    // Run LCD at 48 HZ
    lcd_SetNormalFrameRateControl(LCD_RTN_618 | LCD_NL_DEFAULT);
    os_ClrLCDFull();
    gfx_Begin();
    gfx_ZeroScreen();
    gfx_SetDrawBuffer();

    return ticevid_video_init();
}

void ticevid_draw_cleanup(void) {
    lcd_SetNormalFrameRateControl(LCD_FRCTRL_DEFAULT);
    lcd_Cleanup();
    gfx_End();
    os_ClrHomeFull();
    os_DrawStatusBar();
    os_HomeUp();
    ticevid_video_cleanup();
}

static void ticevid_draw_status(char *text) {
    gfx_FillScreen(0xFF);
    fontlib_ClearWindow();
    fontlib_HomeUp();
    fontlib_DrawString(text);
}

ticevid_result_t ticevid_draw_update(void) {
    ticevid_vbuffer = *gfx_vbuffer;

    switch (ui_state) {
        case TICEVID_UI_MAIN:
            ticevid_draw_status("TICEVID: The USB Video Player\nPress enter to connect to USB device.");
            break;
        case TICEVID_UI_LOADING_VIDEO_SELECT_PRE:
            ticevid_draw_status("Waiting for USB connection...");
            break;
        case TICEVID_UI_LOADING_VIDEO_SELECT:
            ticevid_draw_status("Loading container header...");
            break;
        case TICEVID_UI_LOADING_VIDEO:
            ticevid_draw_status("Loading video...");
            break;
        case TICEVID_UI_TITLE_SELECT:
            ticevid_draw_status("Select title.");
            fontlib_Newline();

            ticevid_container_header_t container = *ticevid_video_container_header;

            for (uint8_t i = 0; i < container.title_count; i++) {
                ticevid_title_t title = *container.title_table[i];

                if (i == ticevid_ui_title_select_index) {
                    fontlib_DrawString(" >");
                } else {
                    fontlib_DrawString("  ");
                }

                fontlib_DrawString(title.name);
                fontlib_Newline();
            }

            fontlib_DrawString("Done.");
            break;
        case TICEVID_UI_PLAYING_PRE:
            gfx_ZeroScreen();
            ui_state = TICEVID_UI_PLAYING;
        case TICEVID_UI_PLAYING:
            EARLY_EXIT(ticevid_video_play_draw());
            break;
    }

    // Half frame rate
    gfx_SwapDraw();
    gfx_BlitScreen();
    gfx_SwapDraw();

    return TICEVID_SUCCESS;
}
