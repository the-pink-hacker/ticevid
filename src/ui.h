#include "error.h"

typedef enum ticevid_ui_state {
    TICEVID_UI_MAIN = 0,
    TICEVID_UI_LOADING_VIDEO_SELECT,
    TICEVID_UI_TITLE_SELECT,
    TICEVID_UI_LOADING_VIDEO,
    TICEVID_UI_PLAYING,
} ticevid_ui_state_t;

extern ticevid_ui_state_t ui_state;

ticevid_result_t ticevid_ui_update(void);
