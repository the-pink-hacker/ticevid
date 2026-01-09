typedef struct usb_state usb_state_t;
#define usb_callback_data_t usb_state_t

#include <stdint.h>
#include <string.h>

#include <graphx.h>
#include <msddrvce.h>
#include <sys/lcd.h>
#include <usbdrvce.h>

#include "error.h"
#include "io.h"
#include "usb.h"

struct usb_state {
    usb_device_t device;
    // Mass storage device
    msd_t msd;
};

static usb_state_t usb_state;

static usb_error_t ticevid_usb_handle_event(
    usb_event_t event,
    void *event_data,
    usb_callback_data_t *state
) {
    switch (event) {
        case USB_DEVICE_DISCONNECTED_EVENT:
            if (state->device) {
                msd_Close(&state->msd);
            }
            state->device = NULL;
            break;
        case USB_DEVICE_CONNECTED_EVENT:
            return usb_ResetDevice(event_data);
        case USB_DEVICE_ENABLED_EVENT:
            state->device = event_data;
            break;
        case USB_DEVICE_DISABLED_EVENT:
            // Retries each time
            return USB_USER_ERROR;
        default:
            break;
    }

    return USB_SUCCESS;
}

// Connects the usb device
static ticevid_result_t ticevid_usb_connect_device(void) {
    usb_error_t result;

    if (usb_state.device != NULL) {
        return TICEVID_SUCCESS;
    }

    do {
        usb_state.device = NULL;
        result = usb_Init(
            ticevid_usb_handle_event,
            &usb_state,
            NULL,
            USB_DEFAULT_INIT_FLAGS
        );

        if (result != USB_SUCCESS) {
            RETURN_ERROR(TICEVID_USB_INIT_ERROR);
        }

        while (result == USB_SUCCESS) {
            if (usb_state.device != NULL) {
                break;
            }

            result = usb_WaitForInterrupt();
        }
    } while (result == USB_USER_ERROR);

    if (result != USB_SUCCESS) {
        RETURN_ERROR(TICEVID_USB_ENABLE_ERROR);
    }

    return TICEVID_SUCCESS;
}

// Opens the usb as a mass storage device
static ticevid_result_t ticevid_usb_setup_msd(void) {
    msd_error_t open_result = msd_Open(
        &usb_state.msd,
        usb_state.device
    );

    if (open_result != MSD_SUCCESS) {
        RETURN_ERROR(TICEVID_MSD_OPEN_ERROR);
    }

    return TICEVID_SUCCESS;
}

void ticevid_usb_init(void) {
    memset(&usb_state, 0, sizeof(usb_state_t));
}

ticevid_result_t ticevid_usb_attempt_connection(void) {
    ticevid_result_t usb_result = ticevid_usb_connect_device();

    if (usb_result != USB_SUCCESS) {
        return usb_result;
    }

    return ticevid_usb_setup_msd();
}

void ticevid_usb_cleanup(void) {
    msd_Close(&usb_state.msd);
    usb_Cleanup();
}

ticevid_result_t ticevid_usb_copy_chunk(uint24_t start_block, uint8_t blocks, uint8_t *buffer) {
    uint24_t amount_read = msd_Read(&usb_state.msd, start_block, blocks, buffer);

    if (amount_read != blocks) {
        msd_Close(&usb_state.msd);
        RETURN_ERROR(TICEVID_MSD_READ_ERROR);
    }

    return TICEVID_SUCCESS;
}

bool ticevid_usb_connected(void) {
    return usb_state.device != NULL;
}
