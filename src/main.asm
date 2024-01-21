.nolist
#include "includes/ti84pce.inc"
.list

.org userMem - 2
.assume ADL = 1

.db tExtTok, tAsm84CeCmp
main:
  call _RunIndicOff
  call _ClrLCDAll
  call lcd_init

  call player_init
exit:
  call _ClrScrnFull
  call lcd_clean_up
  call _HomeUp
  jp _DrawStatusBar

#include "src/gfx.asm"
#include "src/player.asm"
