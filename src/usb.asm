init_usb:
    ld hl, 36106
    push hl

    ld hl, fat_path
    push hl
    push hl

    ld hl, .usb_event_callback
    push hl

    call usb.Init
    pop bc, bc, bc, bc

    call ti.NewLine
    call ti.PutS

    ret

    .usb_event_callback:
        call ti.NewLine
        ld hl, .text
	jp ti.PutS

    .text:
        db "USB event callback.", 0

fat_path:
    db "/", 0
    rb 512 - 2
