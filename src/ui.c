#include <stdint.h>

#include <fileioc.h>
#include <fontlibc.h>

#include "error.h"
#include "ui.h"
#include "usb.h"
#include "io.h"
#include "video.h"

ticevid_ui_state_t ui_state = TICEVID_UI_MAIN;

// Using the sans font because I don't want to make a new font
static const char *TICEVID_FONT_DEFAULT = "SANSFNT";

static fontlib_font_t *ticevid_font_main;

static ticevid_result_t ticevid_font_load(void) {
    uint8_t file = ti_Open(TICEVID_FONT_DEFAULT, "r");

    if (file == 0) {
        return TICEVID_FONT_MISSING;
    }

    fontlib_font_pack_t *font_pack = ti_GetDataPtr(file);
    ti_Close(file);

    fontlib_font_t *font = fontlib_GetFontByIndexRaw(font_pack, 0);

    if (font == NULL) {
        return TICEVID_FONT_INVALID;
    }

    fontlib_SetFont(font, 0);
    ticevid_font_main = font;
    fontlib_SetColors(0, 0xFF);
    fontlib_SetTransparency(true);
    fontlib_SetNewlineOptions(FONTLIB_ENABLE_AUTO_WRAP);
    fontlib_SetWindowFullScreen();

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_ui_update(void) {
    switch (ui_state) {
        case TICEVID_UI_MAIN:
            if (ticevid_font_main == NULL) {
                ticevid_result_t result = ticevid_font_load();
                
                if (result != TICEVID_SUCCESS) {
                    return result;
                }
            }

            if (ticevid_io_pressing_enter()) {
                ui_state = TICEVID_UI_LOADING_VIDEO_SELECT;
            }

            break;
        case TICEVID_UI_LOADING_VIDEO_SELECT:
            if (!ticevid_usb_connected()) {
                ticevid_result_t result = ticevid_usb_attempt_connection();

                if (result != TICEVID_SUCCESS) {
                    return result;
                }

                result = ticevid_video_load_header();

                ui_state = TICEVID_UI_VIDEO_SELECT;

                return result;
            }

            break;
        case TICEVID_UI_VIDEO_SELECT:
            break;
        case TICEVID_UI_LOADING_VIDEO:
            ui_state = TICEVID_UI_PLAYING;
            break;
        case TICEVID_UI_PLAYING:
            return ticevid_video_play_update();
    }

    return TICEVID_SUCCESS;
}
