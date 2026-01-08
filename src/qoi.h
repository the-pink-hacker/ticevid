#include <stdint.h>

#include "error.h"

void ticevid_qoi_init_frame();

ticevid_result_t ticevid_qoi_decode(
    // How many bytes to read from the input buffer
    uint16_t length,
    // The max amount of pixels to write to the output buffer
    uint24_t max_pixels,
    uint8_t *input_buffer
);
