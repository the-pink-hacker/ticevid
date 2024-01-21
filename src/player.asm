player_init:
player_loop:
  call swap_vbuffer
player_loop_exit:
  call _GetCSC
  or a, a
  jr z, player_loop

  ret
