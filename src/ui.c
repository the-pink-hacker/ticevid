#include <stdint.h>

#include <fileioc.h>
#include <fontlibc.h>

#include "error.h"
#include "ui.h"
#include "usb.h"
#include "io.h"
#include "video.h"

ticevid_ui_state_t ui_state = TICEVID_UI_MAIN;

uint8_t ticevid_ui_title_select_index = 0;

static const char *TICEVID_FONT_DEFAULT = "TICEVIDF";

static const uint8_t COLOR_TEXT = 0x00;

static fontlib_font_t *ticevid_font_main;

void ticevid_ui_set_color_default(void) {
    fontlib_SetForegroundColor(COLOR_TEXT);
    fontlib_SetTransparency(true);
}

static ticevid_result_t ticevid_font_load(void) {
    uint8_t file = ti_Open(TICEVID_FONT_DEFAULT, "r");

    if (file == 0) {
        RETURN_ERROR(TICEVID_FONT_MISSING);
    }

    fontlib_font_pack_t *font_pack = ti_GetDataPtr(file);
    ti_Close(file);

    fontlib_font_t *font = fontlib_GetFontByIndexRaw(font_pack, 0);

    if (font == NULL) {
        RETURN_ERROR(TICEVID_FONT_INVALID);
    }

    fontlib_SetFont(font, 0);
    ticevid_font_main = font;
    ticevid_ui_set_color_default();
    fontlib_SetNewlineOptions(FONTLIB_ENABLE_AUTO_WRAP);
    fontlib_SetWindowFullScreen();

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_ui_update(void) {
    ticevid_container_header_t *container = ticevid_video_container_header;

    switch (ui_state) {
        case TICEVID_UI_MAIN:
            if (ticevid_font_main == NULL) {
                EARLY_EXIT(ticevid_font_load());
            }

            if (ticevid_io_pressing_enter()) {
                ui_state = TICEVID_UI_LOADING_VIDEO_SELECT;
            }

            break;
        case TICEVID_UI_LOADING_VIDEO_SELECT:
            if (!ticevid_usb_connected()) {
                EARLY_EXIT(ticevid_usb_attempt_connection());

                ticevid_result_t result = ticevid_video_load_header();

                ui_state = TICEVID_UI_TITLE_SELECT;

                return result;
            }

            break;
        case TICEVID_UI_TITLE_SELECT:

            if (ticevid_io_pressing_enter()) {
                ticevid_title_t *title = container->title_table[ticevid_ui_title_select_index];

                ticevid_video_select_title(title);
                ui_state = TICEVID_UI_LOADING_VIDEO;
                break;
            }

            if (ticevid_io_pressed_up()) {
                if (ticevid_ui_title_select_index == 0) {
                    ticevid_ui_title_select_index = container->title_count - 1;
                } else {
                    ticevid_ui_title_select_index--;
                }
            }

            if (ticevid_io_pressed_down()) {
                // The last index
                if (ticevid_ui_title_select_index == container->title_count - 1) {
                    ticevid_ui_title_select_index = 0;
                } else {
                    ticevid_ui_title_select_index++;
                }
            }
            break;
        case TICEVID_UI_LOADING_VIDEO:
            ui_state = TICEVID_UI_PLAYING;
            break;
        case TICEVID_UI_PLAYING:
            return ticevid_video_play_update();
    }

    return TICEVID_SUCCESS;
}
