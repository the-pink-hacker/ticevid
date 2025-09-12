#include <stdint.h>

#include "error.h"

// Called once at the start of the program
void ticevid_usb_init(void);

// Tries to connect a usb device
// Should be ran each frame durring connection
ticevid_result_t ticevid_usb_attempt_connection(void);

int ticevid_usb_cleanup(void);

ticevid_result_t ticevid_usb_copy_frame(uint24_t frame);

bool ticevid_usb_connected(void);
