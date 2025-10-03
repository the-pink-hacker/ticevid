#include <stdint.h>
#include <msddrvce.h>

#include "error.h"

// Called once at the start of the program
void ticevid_usb_init(void);

// Tries to connect a usb device
// Should be ran each frame durring connection
ticevid_result_t ticevid_usb_attempt_connection(void);

void ticevid_usb_cleanup(void);

ticevid_result_t ticevid_usb_copy_chunk(uint24_t chunk, uint8_t blocks, uint8_t *buffer);

bool ticevid_usb_connected(void);
