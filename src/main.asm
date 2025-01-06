include "include/ti84pceg.inc"
include "include/ez80.inc"
include "include/tiformat.inc"
include "include/macros.inc"
format ti executable "TICEVID"

ticevid:
    call ti.RunIndicOff
    call ti.ClrLCDAll
    call ti.HomeUp

    call load_libload_libraries
    jq nz, failed_to_load_libs

    call usb.init_usb
    jq nz, .exit

    ld hl, .text_2
    call ti.PutS
    
    call ti.GetKey

    assert $ = .exit

    .exit:
        call usbdrvce.Cleanup
        call ti.ClrScrnFull
	call ti.HomeUp
	jp ti.DrawStatusBar

    .text_2:
        string "POST-INIT-USB"

include "src/libload.asm"
include "src/usb.asm"
