#include <ti/screen.h>
#include <graphx.h>

#include "error.h"
#include "qoi.h"
#include "usb.h"

static uint24_t current_frame = 0;
static uint8_t *chunk_buffer;
static bool copied = false;

void ticevid_draw_init(void) {
    os_RunIndicOff();
    os_ClrLCDFull();
    gfx_Begin();
    gfx_ZeroScreen();
    gfx_SetDrawBuffer();
    chunk_buffer = (uint8_t *)malloc(TICEVID_CHUNK_BYTES);
}

void ticevid_draw_cleanup(void) {
    gfx_End();
    os_ClrHomeFull();
    os_DrawStatusBar();
    os_HomeUp();
    free(chunk_buffer);
}

ticevid_result_t ticevid_draw_update(void) {
    ticevid_result_t result;

    if (ticevid_usb_connected()) {
        if (!copied) {
            result = ticevid_usb_copy_chunk(current_frame, chunk_buffer);

            if (result != TICEVID_SUCCESS) {
                return result;
            }

            copied = true;
        }

        gfx_Wait();

        result = ticevid_qoi_decode(7898, chunk_buffer, *gfx_vbuffer);

        if (result != TICEVID_SUCCESS) {
            return result;
        }
    }

    gfx_SwapDraw();

    return TICEVID_SUCCESS;
}
