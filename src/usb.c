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
#include "video.h"

struct usb_state {
    usb_device_t device;
    // Mass storage device
    msd_t msd;
};

static usb_state_t usb_state;

typedef struct msd_state {
    // The async read is fully writen
    bool completed;
    // The error from the callback
    msd_error_t error;
} msd_state_t;

static msd_state_t msd_state;

// Called on completion
static void _msd_async_callback(msd_error_t error, msd_transfer_t *xfer) {
    (void)xfer;
    msd_state.error = error;
    msd_state.completed = true;
}

static msd_transfer_t msd_transfer = {
    .msd = &usb_state.msd,
    .count = TICEVID_BLOCKS_PER_CHUNK,
    .callback = _msd_async_callback
};

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
    memset(&msd_state, 0, sizeof(msd_state_t));
}

ticevid_result_t ticevid_usb_update(void) {
    usb_error_t result = usb_HandleEvents();
    
    if (result == USB_SUCCESS) {
        return TICEVID_SUCCESS;
    } else {
        RETURN_ERROR(TICEVID_USB_EVENT_ERROR);
    }
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

ticevid_result_t ticevid_usb_copy_chunk(uint32_t start_block, uint8_t *buffer) {
    msd_transfer.lba = start_block;
    msd_transfer.buffer = buffer;

    msd_error_t result = msd_ReadAsync(&msd_transfer);

    if (result != MSD_SUCCESS) {
        msd_Close(&usb_state.msd);
        RETURN_ERROR(TICEVID_MSD_READ_ERROR);
    }

    return TICEVID_SUCCESS;
}

ticevid_result_t ticevid_usb_msd_poll(void) {
    if (!msd_state.completed) {
        // No need for RETURN_ERROR
        // Should be handled by the caller
        return TICEVID_MSD_ASYNC_WAIT;
    }

    msd_state.completed = false;

    if (msd_state.error != MSD_SUCCESS) {
        RETURN_ERROR(TICEVID_MSD_READ_ERROR);
    }

    return TICEVID_SUCCESS;
}

bool ticevid_usb_connected(void) {
    return usb_state.device != NULL;
}
