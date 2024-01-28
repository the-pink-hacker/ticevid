include "include/ti84pceg.inc"
include "include/ez80.inc"
include "include/tiformat.inc"
format ti executable "TICEVID"

ticevid:
    call ti.RunIndicOff
    call ti.ClrLCDAll
    call ti.HomeUp
    call load_libload_libraries
    jr nz, failed_to_load_libs

    ld hl, .text
    call ti.PutS

    call ti.GetKey

    call init_usb
    
    call ti.GetKey

    .exit:
        call usb.Cleanup
        call ti.ClrScrnFull
	call ti.HomeUp
	jp ti.DrawStatusBar

    .text:
        db "Hello World!", 0

include "src/libload.asm"
include "src/usb.asm"
