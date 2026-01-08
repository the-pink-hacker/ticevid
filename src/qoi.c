#include <string.h>

#include "draw.h"
#include "qoi.h"

const uint8_t QOI_TAG_LITERAL = 0xFF;
const uint8_t QOI_TAG_DIFF = 0;
const uint8_t QOI_TAG_INDEX = 0b10000000;
const uint8_t QOI_TAG_RUN = 0b11000000;
const uint8_t QOI_TAG_DATA_MASK = 0b00111111;

static uint8_t index[64];
static uint8_t previous_pixel;
static uint8_t *output_buffer;

static uint8_t index_hash(uint8_t value) {
    return value % 64;
}

static void index_insert(uint8_t value) {
    index[index_hash(value)] = value;
}

void ticevid_qoi_init_frame() {
    memset(index, 0, sizeof(index));
    previous_pixel = 0;
    output_buffer = ticevid_vbuffer;
}

ticevid_result_t ticevid_qoi_decode(
    uint24_t length,
    uint24_t max_pixels,
    uint8_t *input_buffer
) {
    uint24_t start = (uint24_t)output_buffer;

    for (uint24_t i = 0; i < length; i++) {
        uint8_t tag = input_buffer[i];

        if ((tag & QOI_TAG_LITERAL) == QOI_TAG_LITERAL) {
            i++;
            uint8_t pixel = input_buffer[i];
            previous_pixel = pixel;
            *output_buffer = pixel;
            output_buffer++;
            index_insert(pixel);
        } else if ((tag & 0b11000000) == QOI_TAG_RUN) {
            uint8_t repeat = (tag & QOI_TAG_DATA_MASK) + 1;

            memset(output_buffer, previous_pixel, repeat);
            output_buffer += repeat;
        } else if ((tag & 0b10000000) == QOI_TAG_DIFF) {
            uint8_t diff = tag & 0b0111111;
            
            uint8_t pixel;

            if (diff <= 63) {
                pixel = previous_pixel - diff - 1;
            } else {
                pixel = previous_pixel + (128 - diff);
            }

            previous_pixel = pixel;
            *output_buffer = pixel;
            output_buffer++;
            index_insert(pixel);
        } else if ((tag & 0b11000000) == QOI_TAG_INDEX) {
            uint8_t hash = tag & QOI_TAG_DATA_MASK;
            uint8_t pixel = index[hash];
            previous_pixel = pixel;
            *output_buffer = pixel;
            output_buffer++;
        } else {
            return TICEVID_QOI_TAG;
        }

        if (output_buffer - start >= max_pixels) {
            return TICEVID_SUCCESS;
        }
    }

    return TICEVID_SUCCESS;
}
