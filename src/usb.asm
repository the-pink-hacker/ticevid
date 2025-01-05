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
        rl 1 ; pointer to array of endpoints
	rb 1 ; device addr and $7F
	rl 1 ; reference count
	rb 1 ; device speed shl 4
	rl 1 ; next device connected to the same hub
	rl 1 ; update pointer to next pointer to self
	rb 1 ; find flags
	rb 1 ; port number of hub this device is connected to
	rl 1 ; first device connected to this hub
	rb 1 ; padding
	rl 1 ; hub this device is connected to
	rl 1 ; user data
	rb 6 ; padding
    .msd.offset := $ - .
    .msd:
        rl 1
	rb 1
	rb 1
	rb 1
	rb 1
	rd 1
	rl 1
	rb 1
	rb 768 ; Buffer

usb.init_usb:
    .retry_loop:
        ; flags
        ld hl, usb.init_flags.DEFAULT_INIT_FLAGS
        push hl
            ; device_descriptors
            ld hl, NULL
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
        jq nz, .error_exit ; if a != 0

    .interrupt_loop:
        ld hl, (usb.global)
        or a, a ; Reset c
        sbc hl, hl
        jq z, .error_exit ; if global.usb == NULL

        call ti.GetCSC
        or a, a
        jq nz, .error_exit

        ld hl, usb.global.msd
        push hl ; &global.msd
            call msddrvce.Close
        pop hl

        call usbdrvce.WaitForInterrupt

        or a, a
        jp z, .interrupt_loop ; if a == success

        cp a, 100
        jp z, .retry_loop ; if a == retry

        ld hl, usb.global.usb
        push hl ; &global.usb
            ld hl, usb.global.msd
            push hl ; &global.msd
                call msddrvce.Open
            pop hl
        pop hl

        or a, a
        jp nz, .error_exit

    .error_exit:
        ld hl, .error_text
        call ti.PutS
        jq ticevid.exit
    
    .error_text:
        db "USB Init Error"

usb.handle_usb_event:
    .global     := 3 ; u24
    .event_data := 6 ; u24
    .event      := 9 ; u8

    ld iy, 0
    add iy, sp

    ld a, (iy + .event)

    .disconnected:
        dec a
        jq nz, .connected

        ld hl, .text_disconnected
        call ti.PutS ; Does iy get preserved???

        ld hl, (iy + .global) ; &global.usb
        or a, a ; Reset c
        ld de, 1
        sbc hl, de
        ld hl, (iy + .global) ; &global.usb
        jq c, .disconnected_return ; if global.usb == NULL
        ld de, usb.global.msd.offset
        add hl, de
        push hl ; &global.msd
            call msddrvce.Close
        pop hl
        ld de, usb.global.msd.offset
        or a, a ; Reset c
        sbc hl, de ; hl = &global.usb
        assert $ = .disconnected_return

    .disconnected_return:
        ld (hl), NULL ; global.usb = NULL

        xor a, a
        ret

    .connected:
        dec a
        jq nz, .disabled

        ld hl, .text_connected
        call ti.PutS

        ld hl, (iy + .event_data)
        push hl
            call usbdrvce.ResetDevice
        pop hl

        ret

    .disabled:
        dec a
        jq nz, .enabled

        ld hl, .text_disabled
        call ti.PutS

        ld a, 100 ; Retry
        ret

    .enabled:
        dec a
        ld a, 0
        ret nz

        ld hl, (iy + .event_data)
        ld (iy + .global), hl

        ld hl, .text_enabled
        call ti.PutS

        ret ; a = 0

    .text_disconnected:
        string "USB Event: Device Disconnected"

    .text_connected:
        string "USB Event: Device Connected"

    .text_disabled:
        string "USB Event: Device Disabled"

    .text_enabled:
        string "USB Event: Device Enabled"
