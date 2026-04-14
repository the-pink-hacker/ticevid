#include <stdint.h>

#include "error.h"

// Preps picture decoding for the next frame
void ticevid_qoi_init_frame(uint24_t pixel_offset);

ticevid_result_t ticevid_qoi_decode(
    // How many bytes to read from the input buffer
    uint16_t length,
    // The amount of pixels that are left to be read
    uint24_t *remaining_pixels,
    uint8_t *input_buffer
);
