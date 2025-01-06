usb.init_flags:
    .USE_C_HEAP := 1 shl 0
    .USE_OS_HEAP := 1 shl 1
    .INIT_FLSZ_1024 := (0 and 3) shl 2
    .INIT_FLSZ_512 := (1 and 3) shl 2
    .INIT_FLSZ_256 := (2 and 3) shl 2
    .INIT_FLSZ_0 := (3 and 3) shl 2
    .INIT_ASST_0 := (0 and 3) shl 8
    .INIT_ASST_1 := (1 and 3) shl 8
    .INIT_ASST_2 := (2 and 3) shl 8
    .INIT_ASST_3 := (3 and 3) shl 8
    .INIT_EOF1_0 := (0 and 3) shl 10
    .INIT_EOF1_1 := (1 and 3) shl 10
    .INIT_EOF1_2 := (2 and 3) shl 10
    .INIT_EOF1_3 := (3 and 3) shl 10
    .INIT_EOF2_0 := (0 and 3) shl 12
    .INIT_EOF2_1 := (1 and 3) shl 12
    .INIT_EOF2_2 := (2 and 3) shl 12
    .INIT_EOF2_3 := (3 and 3) shl 12
    .INIT_UNKNOWN := 1 shl 15
    .DEFAULT_INIT_FLAGS := .USE_OS_HEAP or .INIT_FLSZ_256 or .INIT_ASST_1 or .INIT_EOF1_3 or .INIT_EOF2_0 or .INIT_UNKNOWN

usb.global:
    .usb:
        rl 1
    .msd:
        rl 256

usb.init_usb:
    .retry_loop:
        ; flags
        ld hl, usb.init_flags.DEFAULT_INIT_FLAGS
        push hl
            ; device_descriptors
            or a, a ; reset c
            sbc hl, hl ; hl = NULL
            push hl
                ; data
                ld hl, usb.global
                push hl
                    ; handler
                    ld hl, usb.handle_usb_event
                    push hl
                        call usbdrvce.Init
                    pop hl
                pop hl
            pop hl
        pop hl

        or a, a
        jq nz, .error_exit ; if a != success

    .interrupt_loop:
        ld hl, (usb.global)
        or a, a ; Reset c
        ld de, 1
        sbc hl, de
        jq nc, .retry_loop.exit ; if global.usb != NULL

        call ti.GetCSC
        or a, a
        jq nz, .error_exit

        call usbdrvce.WaitForInterrupt

        or a, a
        jq z, .interrupt_loop ; if a == success

        cp a, 100
        jq z, .retry_loop ; if a == retry

    .retry_loop.exit:

        ld hl, usb.global.usb
        ld de, (hl)
        push de ; &global.usb
            inc hl
            inc hl
            inc hl
            push hl ; &global.msd
                call msddrvce.Open
            pop hl
        pop hl

        or a, a
        ret z
        assert $ = .error_exit

    .error_exit:
        ld hl, .error_text
        call ti.PutS
        call ti.NewLine
        call ti.GetKey

        xor a, a
        inc a
        ret
    
    .error_text:
        string "USB Init Error"

usb.handle_usb_event:
    .global     := 3 ; u24
    .event_data := 6 ; u24
    .event      := 9 ; u8

    push iy
        ld iy, 0
        add iy, sp

        ld a, (iy + .event)

        dec a
        jq z, .disconnected
        dec a
        jq z, .connected
        dec a
        jq z, .disabled
        dec a
        jq z, .enabled
    pop iy

    xor a, a
    ret

    .disconnected:
        ld hl, (iy + .global) ; &global.usb
        pop iy
        push hl
            ld hl, .text_disconnected
            call ti.PutS
            call ti.NewLine
        pop hl

        or a, a ; Reset c
        ld de, 1
        sbc hl, de ; if global.usb == NULL
        jq c, .disconnected_return ; if global.usb == NULL
        ld hl, usb.global.msd
        push hl ; &global.msd
            call msddrvce.Close
        pop hl
        assert $ = .disconnected_return

    .disconnected_return:
        ld hl, usb.global.usb
        ld (hl), NULL

        xor a, a
        ret

    .connected:
        ld hl, (iy + .event_data)
        pop iy
        push hl
            ld hl, .text_connected
            call ti.PutS
            call ti.NewLine
        ;pop hl
        ;push hl
            call usbdrvce.ResetDevice
        pop hl

        ret

    .disabled:
        pop iy
        ld hl, .text_disabled
        call ti.PutS
        call ti.NewLine

        ld a, 100 ; Retry
        ret

    .enabled:
        ld hl, (iy + .event_data)
        ld (iy + .global), hl
        pop iy

        ld hl, .text_enabled
        call ti.PutS
        call ti.NewLine

        xor a, a
        ret

    .text_disconnected:
        string "USB Event: Device Disconnected"

    .text_connected:
        string "USB Event: Device Connected"

    .text_disabled:
        string "USB Event: Device Disabled"

    .text_enabled:
        string "USB Event: Device Enabled"
