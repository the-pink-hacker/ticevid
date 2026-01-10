#include "error.h"

typedef enum ticevid_ui_state {
    TICEVID_UI_MAIN = 0,
    TICEVID_UI_LOADING_VIDEO_SELECT_PRE,
    TICEVID_UI_LOADING_VIDEO_SELECT,
    TICEVID_UI_TITLE_SELECT,
    TICEVID_UI_LOADING_VIDEO,
    TICEVID_UI_PLAYING_PRE,
    TICEVID_UI_PLAYING,
} ticevid_ui_state_t;

extern ticevid_ui_state_t ui_state;

extern uint8_t ticevid_ui_title_select_index;

// Sets the font colors to the default colors
void ticevid_ui_text_color_default(void);

ticevid_result_t ticevid_ui_update(void);
