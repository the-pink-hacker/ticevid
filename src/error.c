#include <ti/sprintf.h>

#include "error.h"
#include "io.h"

// TODO: Only add to debug builds

static char ticevid_error_buffer[64];
static char *ticevid_error_file;
static uint24_t ticevid_error_line = -1;

void ticevid_error_print(char *text) {
    ticevid_io_println(text);
    boot_sprintf(
        ticevid_error_buffer,
        "%s:%u",
        ticevid_error_file,
        (unsigned int)ticevid_error_line
    );
    ticevid_io_println(ticevid_error_buffer);
}

void ticevid_error_set_file_line(char *file, uint24_t line) {
    ticevid_error_file = file;
    ticevid_error_line = line;
}
