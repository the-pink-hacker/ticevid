include "include/ti84pceg.inc"
include "include/ez80.inc"
include "include/tiformat.inc"
format ti executable "TICEVID"

ticevid:
    call ti.ClrScrnFull
    call ti.HomeUp

    ld hl, .text
    call ti.PutS
  
    call ti.GetKey

.exit:
    call ti.ClrScrnFull
    ret

.text:
    db "Hello World!", 0
