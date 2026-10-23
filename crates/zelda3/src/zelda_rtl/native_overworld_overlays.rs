//! Retire overlay stores without importing state or constructing a return clock.
use super::*;
use crate::game_state::constants::{NUM_MEMORIZED_TILES};
impl ZeldaState {
    pub(super) fn native_overworld_screen_build_store(&mut self,address:u32,value:u16,width:u8)->Result<(),usize> {
        let address=if address&0x400000==0 && address&65535<0x2000 {address&65535} else {address};
        let offset=if (0x7e0000..0x800000).contains(&address) {address as usize-0x7e0000} else {address as usize};
        for (byte, value) in value.to_le_bytes().into_iter().take(usize::from(width)).enumerate() {
            self.write_wram_byte(offset + byte, value);
        }
        match offset {
            0x2000..=0x5fff=>{
                for byte in 0..usize::from(width) {
                    let at=(offset+byte)&!1;let word=read_le_u16(&self.ram,at);
                    self.game_state.dungeon.room_tilemaps.write_map_tile_word(at,word);
                }
            }
            0x4ac..=0x4ad=>self.game_state.memorized_tiles.set_count(read_le_u16(&self.ram,NUM_MEMORIZED_TILES)),
            0xf800..=0xf9ff | 0xfa00..=0xfbff=>{
                for byte in 0..usize::from(width) {
                    let at=(offset+byte)&!1;let word=read_le_u16(&self.ram,at);
                    if at<0xfa00 {self.game_state.memorized_tiles.set_entry_addr(at-0xf800,word);}
                    else {self.game_state.memorized_tiles.set_entry_value(at-0xfa00,word);}
                }
            }
            0x10=>self.game_state.frame.set_main_module(value as u8),
            0xb0=>self.game_state.frame.set_subsubmodule(value as u8),
            _=>return self.native_overworld_overlays_store(address,value,width),
        }
        for (byte, value) in value.to_le_bytes().into_iter().take(usize::from(width)).enumerate() {
            self.write_wram_byte(offset + byte, value);
        }
        Ok(())
    }
    pub(super) fn native_overworld_overlays_store(&mut self,address:u32,value:u16,width:u8)->Result<(),usize> {
        let address=if address&0x400000==0 && address&65535<0x2000 {address&65535} else {address};
        let offset=if (0x7e0000..0x800000).contains(&address) {address as usize-0x7e0000} else {address as usize};
        assert!(matches!(width,1|2));
        for (byte, value) in value.to_le_bytes().into_iter().take(usize::from(width)).enumerate() {
            self.write_wram_byte(offset + byte, value);
        }
        if (0x500..0x540).contains(&offset) {
            for byte in 0..usize::from(width) {
                let at=offset+byte;
                self.game_state.world.transient.overworld_map16_stripe[(at-0x500)/2]=read_le_u16(&self.ram,at&!1);
            }
            return Ok(());
        }
        if matches!(offset,0..=0xf|0xc8..=0xce|0x1d0..=0x1ff|0x4000..=0x5fff|0x12000..=0x144ff) {
            // Stack, graphics operands, source cache, map16 staging and map8
            // upload buffers are canonical live RAM during this caller.
            return Ok(());
        }
        for byte in 0..usize::from(width) {
            let at=offset+byte;let v=(value>>(byte*8)) as u8;
            let owned=self.game_state.world.write_overlay_byte(at,v)
                | self.game_state.world.write_properties_byte(at,v)
                | self.game_state.world.write_entrance_byte(at,v)
                | self.game_state.display.ppu_scroll_copy.write_entrance_byte(at,v)
                | self.game_state.display.write_load_parent_byte(at,v);
            if owned {continue;}
            match at {
                0x11=>self.game_state.frame.set_submodule(v),
                0x12d=>self.game_state.system_signals.set_ambient_sound_effect(v),
                0x17=>self.set_pending_nmi_subroutine(v),
                0x710=>self.set_core_update_disable_flag(v),
                _=>return Err(at),
            }
        }
        for (byte, value) in value.to_le_bytes().into_iter().take(usize::from(width)).enumerate() {
            self.write_wram_byte(offset + byte, value);
        }
        Ok(())
    }
}
