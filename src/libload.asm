; returns z if loaded, nz if not loaded
load_libload_libraries:
    jr .tryfind

    .inram:
        call ti.Arc_Unarc

    .tryfind:
        ld hl, libload_name
        call ti.Mov9ToOP1
        call ti.ChkFindSym
        jr c, .notfound
        call ti.ChkInRam
        jr z, .inram
        ld hl, 9 + 3 + libload_name.len
        add hl, de
        ld a, (hl)
        cp a, $1F
        jr c, .notfound
        dec hl
        dec hl
        ld de, relocations
        ld bc, .notfound
        push bc
        ld bc, $AA55AA
        jp (hl)

    .notfound:
        xor a, a
        inc a
        ret

failed_to_load_libs:
    ld hl, .text
    jp ti.PutS

    .text:
        string "Failed to load libs."


relocations:
libload_libload:
    libload_header "LibLoad", 31

msddrvce:
    libload_header "MSDDRVCE", 1

    libload_func .Open, 0
    libload_func .Close, 1

graphx:
    libload_header "GRAPHX", 12

    libload_func .Begin, 0
    libload_func .End, 1

usbdrvce:
    libload_header "USBDRVCE", 0
    
    libload_func .Init, 0
    libload_func .Cleanup, 1
    libload_func .WaitForInterrupt, 5
    libload_func .ResetDevice, 13
    
    xor a, a      ; return z (loaded)
    pop hl      ; pop error return
    ret

libload_name:
    db ti.AppVarObj, "LibLoad", 0
    
    .len := $ - .
