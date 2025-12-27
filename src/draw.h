#include <stdint.h>

#include "error.h"

extern uint8_t *ticevid_vbuffer;

ticevid_result_t ticevid_draw_init(void);

void ticevid_draw_cleanup(void);

ticevid_result_t ticevid_draw_update(void);
