//! Pinned Snes9x OAM address, flip, and byte storage for source-ordered CPU/DMA access.
//! The translated renderer's word-oriented OAM remains a separate presentation owner.

use crate::source_obj::{range_time_flags, OBJ_LINES};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SourceOamPort {
    data: Vec<u8>,
    address: u16,
    saved_address: u16,
    flip: bool,
    priority_rotation: bool,
    first_sprite: u8,
    write_register: u16,
    address_low: u8,
    address_high: u8,
    v_counter: u16,
    registers: Vec<u8>,
    cgram: Vec<u16>,
    cgadd: u8,
    cgflip: bool,
    cgsaved: u8,
    size_select: u8,
    interlace_obj: bool,
    screen_height: u16,
    forced_blank: bool,
    odd_field: bool,
    current_line: u16,
    previous_line: u16,
    end_y: u16,
    obj_changed: bool,
    obj_line_flags: Vec<u8>,
    range_time_over: u8,
}

impl SourceOamPort {
    pub(crate) fn reset() -> Self {
        Self {
            data: vec![0; 0x220],
            address: 0,
            saved_address: 0,
            flip: false,
            priority_rotation: false,
            first_sprite: 0,
            write_register: 0,
            address_low: 0,
            address_high: 0,
            v_counter: 0,
            registers: {
                let mut registers = vec![0; 0x34];
                registers[0x26] = 1;
                registers[0x28] = 1;
                registers
            },
            cgram: vec![0; 256],
            cgadd: 0,
            cgflip: false,
            cgsaved: 0,
            size_select: 0,
            interlace_obj: false,
            screen_height: 224,
            forced_blank: true,
            odd_field: false,
            current_line: 0,
            previous_line: 0,
            end_y: 0,
            obj_changed: true,
            obj_line_flags: vec![0; OBJ_LINES],
            range_time_over: 0,
        }
    }

    pub(crate) fn set_v_counter(&mut self, value: u16) {
        self.v_counter = value;
    }

    pub(crate) fn render_line(&mut self, scanline: u16, odd_field: bool) {
        self.v_counter = scanline;
        self.odd_field = odd_field;
        if (1..=self.screen_height).contains(&scanline) {
            self.current_line = scanline;
        }
    }

    pub(crate) fn enter_scanline(&mut self, scanline: u16) {
        self.v_counter = scanline;
        if scanline == 0 {
            self.range_time_over = 0;
        } else if scanline == 1 {
            self.previous_line = 0;
            self.current_line = 0;
        } else if scanline == self.screen_height + 1 {
            self.flush_redraw();
            if !self.forced_blank {
                self.address = self.saved_address;
                self.flip = false;
                self.update_first_sprite();
            }
        }
    }

    pub(crate) fn stat77_flags(&mut self) -> u8 {
        self.flush_redraw();
        self.range_time_over
    }

    fn flush_redraw(&mut self) {
        if self.previous_line == self.current_line {
            return;
        }
        if self.obj_changed || self.interlace_obj {
            self.obj_line_flags = range_time_flags(
                &self.data,
                self.size_select,
                self.first_sprite,
                self.priority_rotation,
                self.address,
                self.flip,
                self.interlace_obj,
                self.odd_field,
            );
            self.obj_changed = false;
        }
        self.range_time_over |= self.obj_line_flags[usize::from(self.end_y)];
        self.end_y = self
            .current_line
            .saturating_sub(1)
            .min(self.screen_height - 1);
        self.previous_line = self.current_line;
    }

    pub(crate) fn write(&mut self, register: u8, value: u8) {
        let restore_oam_at_vblank =
            register == 0x00 && self.registers[0] & 0x80 != 0 && self.v_counter == 225;
        if usize::from(register) < self.registers.len() {
            let old = self.registers[usize::from(register)];
            let changed = old != value;
            let flush = match register {
                0x00..=0x01 | 0x05..=0x0c | 0x1a | 0x23..=0x32 => changed,
                _ => false,
            };
            if flush {
                self.flush_redraw();
            }
            if register == 0x33 && changed {
                // ppu.cpp flushes a pseudo-hires change before ScreenHeight,
                // then flushes an interlace change after ScreenHeight.
                if (old ^ value) & 8 != 0 {
                    self.flush_redraw();
                }
                self.screen_height = if value & 4 != 0 { 239 } else { 224 };
                if (old ^ value) & 3 != 0 {
                    self.flush_redraw();
                }
            }
            if register == 0x00 && changed && (old ^ value) & 0x80 != 0 {
                self.forced_blank = value & 0x80 != 0;
            }
            self.registers[usize::from(register)] = value;
        }
        match register {
            0x00 if restore_oam_at_vblank => {
                self.address = self.saved_address;
                self.flip = false;
                self.update_first_sprite();
            }
            0x01 => {
                self.size_select = value >> 5;
                self.obj_changed = true;
            }
            0x02 => {
                self.address_low = value;
                self.address = (u16::from(self.address_high & 1) << 8) | u16::from(value);
                self.saved_address = self.address;
                self.flip = false;
                self.update_first_sprite();
            }
            0x03 => {
                self.address_high = value;
                self.address = (u16::from(value & 1) << 8) | u16::from(self.address_low);
                self.saved_address = self.address;
                self.priority_rotation = value & 0x80 != 0;
                self.flip = false;
                self.update_first_sprite();
                self.obj_changed = true;
            }
            0x04 => {
                if !self.flip {
                    self.write_register = (self.write_register & 0xff00) | u16::from(value);
                }
                if self.address & 0x100 != 0 {
                    let index = usize::from(self.address & 0x10f) * 2 + usize::from(self.flip);
                    if self.data[index] != value {
                        self.flush_redraw();
                        self.obj_changed = true;
                    }
                    self.data[index] = value;
                } else if self.flip {
                    self.write_register = (self.write_register & 0x00ff) | (u16::from(value) << 8);
                    let index = usize::from(self.address) * 2;
                    if self.data[index..index + 2] != self.write_register.to_le_bytes() {
                        self.flush_redraw();
                        self.obj_changed = true;
                    }
                    self.data[index..index + 2].copy_from_slice(&self.write_register.to_le_bytes());
                }
                self.advance_flip();
            }
            0x21 => {
                self.cgadd = value;
                self.cgflip = false;
            }
            0x22 => {
                if self.cgflip {
                    let color = u16::from(self.cgsaved) | (u16::from(value & 0x7f) << 8);
                    if self.cgram[usize::from(self.cgadd)] != color {
                        self.flush_redraw();
                        self.cgram[usize::from(self.cgadd)] = color;
                    }
                    self.cgadd = self.cgadd.wrapping_add(1);
                } else {
                    self.cgsaved = value;
                }
                self.cgflip = !self.cgflip;
            }
            0x33 => {
                if self.interlace_obj != (value & 2 != 0) {
                    self.obj_changed = true;
                }
                self.interlace_obj = value & 2 != 0;
            }
            _ => {}
        }
    }

    pub(crate) fn read(&mut self) -> u8 {
        let index = if self.address & 0x100 != 0 {
            usize::from(self.address & 0x10f) * 2 + usize::from(self.flip)
        } else {
            usize::from(self.address) * 2 + usize::from(self.flip)
        };
        let value = self.data[index];
        self.advance_flip();
        value
    }

    fn advance_flip(&mut self) {
        self.flip = !self.flip;
        if !self.flip {
            self.address = self.address.wrapping_add(1) & 0x1ff;
            self.update_first_sprite();
        }
    }

    fn update_first_sprite(&mut self) {
        let first = if self.priority_rotation {
            ((self.address & 0xfe) >> 1) as u8
        } else {
            0
        };
        if self.first_sprite != first {
            self.first_sprite = first;
            self.obj_changed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceOamPort;

    #[test]
    fn low_table_buffers_first_write_but_high_table_writes_each_byte() {
        let mut port = SourceOamPort::reset();
        port.write(0x04, 0x12);
        port.write(0x02, 0);
        assert_eq!(port.read(), 0);
        port.write(0x02, 0);
        port.write(0x04, 0x12);
        port.write(0x04, 0x34);
        port.write(0x02, 0);
        assert_eq!(port.read(), 0x12);
        assert_eq!(port.read(), 0x34);

        port.write(0x03, 1);
        port.write(0x04, 0x56);
        port.write(0x02, 0);
        assert_eq!(port.read(), 0x56);
        assert_eq!(port.read(), 0);
    }

    #[test]
    fn forced_blank_release_at_vblank_restores_saved_address_and_flip() {
        let mut port = SourceOamPort::reset();
        port.write(0x02, 7);
        port.write(0x04, 0xaa);
        port.write(0x04, 0xbb);
        assert_eq!(port.address, 8);
        port.set_v_counter(225);
        port.write(0x00, 0x80);
        port.write(0x00, 0x0f);
        assert_eq!(port.address, 7);
        assert!(!port.flip);
    }

    #[test]
    fn stat77_flushes_only_rendered_lines_and_retains_source_end_y_order() {
        let mut port = SourceOamPort::reset();
        for sprite in 0..128 {
            port.data[sprite * 4 + 1] = 240;
        }
        for sprite in 0..33 {
            port.data[sprite * 4 + 1] = 5;
        }
        port.render_line(6, false);
        assert_eq!(port.stat77_flags(), 0);
        assert_eq!(port.end_y, 5);
        port.render_line(7, false);
        assert_eq!(port.stat77_flags(), 0x40);
        port.enter_scanline(0);
        assert_eq!(port.stat77_flags(), 0);
    }
}
