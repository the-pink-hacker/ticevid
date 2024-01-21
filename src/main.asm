.nolist
#include "includes/ti84pce.inc"
.list

.org userMem - 2
.assume ADL = 1

.db tExtTok, tAsm84CeCmp
main:
  call _RunIndicOff
  call _ClrLCDAll

  call _HomeUp
  ld hl, TextHelloWorld
  call _PutS
exit:
  call _ClrScrnFull
  call _HomeUp
  jp _DrawStatusBar

TextHelloWorld:
  .db "Hello, World!"
