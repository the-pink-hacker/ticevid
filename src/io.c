#include <keypadc.h>
#include <ti/getcsc.h>
#include <ti/screen.h>

static uint8_t keypad_pressed[8];

void ticevid_io_println(char *str) {
    os_PutStrFull(str);
    os_NewLine();
}

bool ticevid_io_pressing_enter(void) {
    return kb_Data[6] & kb_Enter;
}

bool ticevid_io_pressing_exit(void) {
    return kb_Data[6] & kb_Clear;
}

bool ticevid_io_pressing_up(void) {
    return kb_Data[7] & kb_Up;
}

bool ticevid_io_pressing_down(void) {
    return kb_Data[7] & kb_Down;
}

bool ticevid_io_pressing_left(void) {
    return kb_Data[7] & kb_Left;
}

bool ticevid_io_pressing_right(void) {
    return kb_Data[7] & kb_Right;
}

bool ticevid_io_pressed_up(void) {
    return !(keypad_pressed[7] & kb_Up) && ticevid_io_pressing_up();
}

bool ticevid_io_pressed_down(void) {
    return !(keypad_pressed[7] & kb_Down) && ticevid_io_pressing_down();
}

bool ticevid_io_pressed_left(void) {
    return !(keypad_pressed[7] & kb_Left) && ticevid_io_pressing_left();
}

bool ticevid_io_pressed_right(void) {
    return !(keypad_pressed[7] & kb_Right) && ticevid_io_pressing_right();
}

void ticevid_io_update(void) {
    keypad_pressed[7] = kb_Data[7];
    kb_Scan();
}

void ticevid_io_cleanup(void) {
    kb_Reset();
}
