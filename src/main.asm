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
    jr nz, failed_to_load_libs

    call usb.init_usb
    
    call ti.GetKey

    assert $ = .exit

    .exit:
        call usbdrvce.Cleanup
        call ti.ClrScrnFull
	call ti.HomeUp
	jp ti.DrawStatusBar

include "src/libload.asm"
include "src/usb.asm"
