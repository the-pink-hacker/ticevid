// Prints with a new line
void ticevid_io_println(char *str);

// Is the user pressing enter
bool ticevid_io_pressing_enter(void);

// Is the user pressing exit
bool ticevid_io_pressing_exit(void);

bool ticevid_io_pressing_up(void);
bool ticevid_io_pressing_down(void);
bool ticevid_io_pressing_left(void);
bool ticevid_io_pressing_right(void);

// Only on first frame
bool ticevid_io_pressed_up(void);
bool ticevid_io_pressed_down(void);
bool ticevid_io_pressed_left(void);
bool ticevid_io_pressed_right(void);

// Should be ran at the beginning of each frame
void ticevid_io_update(void);

// Needs to be ran before exiting the program
void ticevid_io_cleanup(void);
