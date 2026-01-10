#include <stdint.h>

#include "error.h"

// Preps picture decoding for the next frame
void ticevid_qoi_init_frame(uint24_t pixel_offset);

ticevid_result_t ticevid_qoi_decode(
    // How many bytes to read from the input buffer
    uint16_t length,
    // The max amount of pixels to write to the output buffer
    uint24_t max_pixels,
    uint8_t *input_buffer
);
