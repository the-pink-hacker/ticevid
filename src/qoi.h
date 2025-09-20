#include <stdint.h>

#include "error.h"

ticevid_result_t ticevid_qoi_decode(
    uint24_t length,
    uint8_t *input_buffer,
    uint8_t *output_buffer
);
