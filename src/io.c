#include <ti/getcsc.h>
#include <ti/screen.h>

#define TICEVID_KEY_ENTER sk_Enter
#define TICEVID_KEY_EXIT sk_Clear

void ticevid_io_println(char *str)
{
    os_PutStrFull(str);
    os_NewLine();
}

bool ticevid_io_pressing_enter(void) {
    return os_GetCSC() == TICEVID_KEY_ENTER;
}

bool ticevid_io_pressing_exit(void) {
    return os_GetCSC() == TICEVID_KEY_EXIT;
}
