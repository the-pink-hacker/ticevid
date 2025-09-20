#include <stdint.h>
#include <string.h>

#include "error.h"

const uint8_t QOI_TAG_LITERAL = 0xFF;
const uint8_t QOI_TAG_DIFF = 0;
const uint8_t QOI_TAG_INDEX = 0b10000000;
const uint8_t QOI_TAG_RUN = 0b11000000;
const uint8_t QOI_TAG_DATA_MASK = 0b00111111;

static uint8_t index[64];

static uint8_t index_hash(uint8_t value) {
    return value % 64;
}

static void index_insert(uint8_t value) {
    index[index_hash(value)] = value;
}

ticevid_result_t ticevid_qoi_decode(
    uint24_t length,
    uint8_t *input_buffer,
    uint8_t *output_buffer
) {
    uint8_t previous_pixel = 0;
    memset(index, 0, sizeof(index));

    for (uint24_t i = 0; i < length; i++) {
        uint8_t tag = input_buffer[i];

        if ((tag & QOI_TAG_LITERAL) == QOI_TAG_LITERAL) {
            i++;
            uint8_t pixel = input_buffer[i];
            previous_pixel = pixel;
            *output_buffer = pixel;
            output_buffer++;
            index_insert(pixel);
            continue;
        }

        if ((tag & 0b11000000) == QOI_TAG_RUN) {
            uint8_t repeat = (tag & QOI_TAG_DATA_MASK) + 1;

            memset(output_buffer, previous_pixel, repeat);
            output_buffer += repeat;

            continue;
        }

        if ((tag & 0b10000000) == QOI_TAG_DIFF) {
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
            continue;
        }

        if ((tag & 0b11000000) == QOI_TAG_INDEX) {
            uint8_t hash = tag & QOI_TAG_DATA_MASK;
            uint8_t pixel = index[hash];
            previous_pixel = pixel;
            *output_buffer = pixel;
            output_buffer++;
            continue;
        }
    }

    return TICEVID_SUCCESS;
}
