#include <stdint.h>
#include <msddrvce.h>

#include "error.h"

// Called once at the start of the program
void ticevid_usb_init(void);

ticevid_result_t ticevid_usb_update(void);

// Tries to connect a usb device
// Should be ran each frame durring connection
ticevid_result_t ticevid_usb_attempt_connection(void);

void ticevid_usb_cleanup(void);

ticevid_result_t ticevid_usb_copy_chunk(uint32_t start_block, uint8_t *buffer);

// Check if read is completed
// Not finished on TICEVID_MSD_ASYNC_WAIT
ticevid_result_t ticevid_usb_msd_poll(void);

bool ticevid_usb_connected(void);
