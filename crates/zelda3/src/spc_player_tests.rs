use super::*;

#[test]
fn copy_variables_to_and_from_spc_ram_uses_c_addresses() {
    let p = spc_player_create();
    let p_ref = unsafe { &mut *p };
    p_ref.new_value_from_snes = [1, 2, 3, 4];
    p_ref.port_to_snes = [5, 6, 7, 8];
    p_ref.counter_sf0c = 0x9a;
    p_ref.tempo = 0x1234;
    p_ref.echo_volume_right = 0x4567;
    p_ref.port1_current_bit = 0x80;
    p_ref.channel[2].pattern_order_ptr_for_chan = 0xabcd;
    p_ref.channel[2].note_ticks_left = 0x44;
    p_ref.channel[2].instrument_pitch_base = 0x5678;
    p_ref.channel[2].pan_value = 0x1357;
    p_ref.channel[2].sfx_pan = 0xc0;

    spc_player_copy_variables_to_ram(p);
    assert_eq!(&p_ref.ram[0x0000..0x0004], &[1, 2, 3, 4]);
    assert_eq!(&p_ref.ram[0x0004..0x0008], &[5, 6, 7, 8]);
    assert_eq!(p_ref.ram[0x000c], 0x9a);
    assert_eq!(word(&p_ref.ram, 0x0052), 0x1234);
    assert_eq!(word(&p_ref.ram, 0x0062), 0x4567);
    assert_eq!(p_ref.ram[0x03e0], 0x80);
    assert_eq!(word(&p_ref.ram, 0x0030 + 2 * 2), 0xabcd);
    assert_eq!(p_ref.ram[0x0070 + 2 * 2], 0x44);
    assert_eq!(word(&p_ref.ram, 0x0220 + 2 * 2), 0x5678);
    assert_eq!(word(&p_ref.ram, 0x0330 + 2 * 2), 0x1357);
    assert_eq!(p_ref.ram[0x03d0 + 2 * 2], 0xc0);

    p_ref.ram[0x0052] = 0xef;
    p_ref.ram[0x0053] = 0xbe;
    p_ref.ram[0x0070 + 2 * 2] = 0x55;
    p_ref.ram[0x03d0 + 2 * 2] = 0x40;
    spc_player_copy_variables_from_ram(p);
    assert_eq!(p_ref.tempo, 0xbeef);
    assert_eq!(p_ref.channel[2].note_ticks_left, 0x55);
    assert_eq!(p_ref.channel[2].sfx_pan, 0x40);

    spc_player_destroy(p);
}
