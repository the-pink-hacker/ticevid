#include "draw.h"
#include "error.h"
#include "io.h"
#include "ui.h"

static ticevid_result_t ticevid_update_loop(void) {
    ticevid_io_update();

    if (ticevid_io_pressing_exit()) {
        RETURN_ERROR(TICEVID_USER_EXIT);
    }

    EARLY_EXIT(ticevid_ui_update());

    return ticevid_draw_update();
}

ticevid_result_t ticevid_update_start(void) {
    ticevid_result_t result;

    do {
        result = ticevid_update_loop();
    } while (result == TICEVID_SUCCESS);

    return result;
}
