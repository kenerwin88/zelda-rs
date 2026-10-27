//! Shops, minigame prizes, collectible items, fairies, and item receipts.

use crate::zelda_rtl::sprite::{
    SpriteDamageFromEntry, SpriteDamageToEntry, SpriteInactiveEntry, SpriteMoveEntry,
    SpriteMoveZEntry, SpritePausedEntry, SpriteShadowEntry, SpriteSingleLargeEntry,
    SpriteSingleSmallEntry, SpriteSpawnInfo, SpriteTileCollisionEntry,
};
use super::*;
use crate::types::sign8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_ShopItem(int k) {  // 9ef4ce — sprite_main.c:25333
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_shop_item(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).subtype2().wrapping_sub(7) as usize) * 5;
        self.sprite_draw_multiple_player_deferred(
            k,
            &SHOP_KEEPER_ITEM_WITH_PRICE_DRAW_FRAMES[base..base + 5],
        );
    }

    // -----------------------------------------------------------------------
    // void MagicPowderItem_Draw(int k) {  // 85f67b
    //   Two deferred large tiles.
    // }
    pub(super) fn magic_powder_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &MAGIC_POWDER_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void GreenPotionItem_Draw(int k) {  // 85f718
    //   Bottle body and green price/label tiles.
    // }
    pub(super) fn green_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &GREEN_POTION_ITEM_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void BluePotionItem_Draw(int k) {  // 85f7bd
    //   Bottle body and blue price/label tiles.
    // }
    pub(super) fn blue_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &BLUE_POTION_ITEM_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void RedPotionItem_Draw(int k) {  // 85f86d
    //   Bottle body and red price/label tiles.
    // }
    pub(super) fn red_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &RED_POTION_ITEM_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void Sprite_E9_PotionShop(int k) {  // 85f633
    pub(super) fn sprite_e9_potion_shop(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_magic_shop_assistant_main(k),
            1 => self.sprite_bag_of_powder(k),
            2 => self.sprite_green_cauldron(k),
            3 => self.sprite_blue_cauldron(k),
            4 => self.sprite_red_cauldron(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_F2_MedallionTablet(int k) {  // 86c00d
    pub(super) fn sprite_f2_medallion_tablet(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.medallion_tablet_main(k),
            1 => self.sprite_dust_cloud(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BagOfPowder(int k) {  // 85f644
    pub(super) fn sprite_bag_of_powder(&mut self, k: usize) {
        self.magic_powder_item_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        // ROM $05:F64E
        if !self.sprite_check_damage_same_layer_long_marked(k)
            || (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0
        {
            return;
        }
        self.link_cancel_dash();
        self.follower_link_state_mut().set_item_receipt_method(0);
        if self
            .link_receive_item_from(
                0x0d,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::PotionShopPowder,
                },
            )
            .is_suspended()
        {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_GreenCauldron(int k) {  // 85f68e
    pub(super) fn sprite_green_cauldron(&mut self, k: usize) {
        self.green_potion_item_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.potion_cauldron_purchase(k, 60, 0x2f);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BlueCauldron(int k) {  // 85f72b
    pub(super) fn sprite_blue_cauldron(&mut self, k: usize) {
        self.blue_potion_item_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.potion_cauldron_purchase(k, 160, 0x30);
    }

    // -----------------------------------------------------------------------
    // void Sprite_RedCauldron(int k) {  // 85f7d0
    pub(super) fn sprite_red_cauldron(&mut self, k: usize) {
        self.red_potion_item_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.potion_cauldron_purchase(k, 120, 0x2e);
    }

    fn potion_cauldron_purchase(&mut self, k: usize, price: u16, item: u8) {
        self.sprite_behave_as_barrier(k);
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        if !self.potion_cauldron_check_bottles() {
            if self.sprite_show_message_on_contact(k, 0x4f) & 0x100 != 0 {
                self.potion_cauldron_go_beep(k);
            }
            return;
        }
        if !self.sprite_check_damage_to_link_same_layer(k)
            || (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0
        {
            return;
        }
        if self.game_state.inventory.player_resources.rupees_goal() < price {
            self.sprite_show_message_unconditional(0x17c);
            self.potion_cauldron_go_beep(k);
            return;
        }
        if self.sprite_find_empty_bottle() < 0 {
            self.sprite_show_message_unconditional(0x50);
            self.potion_cauldron_go_beep(k);
            return;
        }
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x1d);
        let value = 64;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let rupees = self
            .game_state
            .inventory
            .player_resources
            .rupees_goal()
            .wrapping_sub(price);
        self.player_resources_mut().set_rupees_goal(rupees);
        self.follower_link_state_mut().set_item_receipt_method(0);
        let _ = self.link_receive_item_from(
            item,
            0,
            ItemReceiptCaller::SpriteMainDirect {
                sprite_slot: k as u8,
                suffix: SpriteMainItemReceiptSuffix::PotionCauldron,
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_MagicShopAssistant_Main(int k) {  // 85f893
    pub(super) fn sprite_magic_shop_assistant_main(&mut self, k: usize) {
        self.shopkeeper_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.player_resources_mut().set_heart_filler(160);
            let value = 0;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        let value = self.game_state.frame.frame_counter >> 5 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let msg = if self.game_state.inventory.items.bottle(0) >= 2
            || self.game_state.inventory.items.bottle(1) >= 2
            || self.game_state.inventory.items.bottle(2) >= 2
            || self.game_state.inventory.items.bottle(3) >= 2
            || !self.game_state.world.region.flag_overworld_area_changed()
        {
            0x4e
        } else {
            0x4d
        };
        if self.sprite_show_solicited_message(k, msg) & 0x100 != 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void FakeSword_Draw(int k) {  // 85eee6
    //   Sprite_DrawMultiplePlayerDeferred(k, kFakeSword_Dmd, 2, NULL);
    // }
    pub(super) fn fake_sword_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &FAKE_SWORD_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void Sprite_E7_Mushroom(int k) {  // 85ee78
    pub(super) fn sprite_e7_mushroom(&mut self, k: usize) {
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        if self.sprite_check_if_link_is_busy() {
            return;
        }

        if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_DRAW)
            && self.game_state.frame.submodule != 0
        {
            return;
        }

        if self.sprite_check_damage_to_link_same_layer(k) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_link_state_mut().set_item_receipt_method(0);
            // Nothing follows the source call; the typed suffix is empty.
            let _ = self.link_receive_item_from(
                0x29,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::Mushroom,
                },
            );
        } else if (self.game_state.frame.frame_counter & 0x1f) == 0 {
            self.sprite_slot_view_mut(k).xor_oam_flags(0x40);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_E8_FakeSword(int k) {  // 85eeaf
    pub(super) fn sprite_e8_fake_sword(&mut self, k: usize) {
        self.fake_sword_draw(k);
        if self.sprite_return_if_paused(k, SpritePausedEntry::Rom06D9F3) {
            return;
        }
        if self.sprite_slot_view(k).draw_work_byte_3() == 3 {
            if self.sprite_slot_view(k).c() == 0 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_c(value);
                self.sprite_show_message_unconditional(0x6f);
            }
        } else {
            // $05:EECE JSR $05:F9ED: complete body, caller opcode unchanged.
            self.sprite_move_xy(k, SpriteMoveEntry::Rom05FA00);
            self.thrown_sprite_tile_and_sprite_interaction(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_E4_SmallKey(int k) {  // 86d032
    pub(super) fn sprite_e4_small_key(&mut self, k: usize) {
        let idx = usize::from(self.sprite_slot_view(k).die_action());
        if (self.game_state.dungeon.savegame_state.savegame_state_bits()
            & (ABSORB_BIG_KEY_MASKS_DRAW[idx] << 8))
            != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        self.sprite_draw_ripple_if_in_water(k);
        if self.sprite_draw_absorbable_transient(k, false) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HeartContainer(int k) {  // 85ef47
    pub(super) fn sprite_heart_container(&mut self, k: usize) {
        if self.game_state.inventory.save_progress.palace_index_x2() == 26 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        let value = self.sprite_slot_view(k).g();
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).g() == 0 {
            self.decode_animated_sprite_tile_variable(3);
            self.sprite_get16_bit_coords(k);
            let value = 1;
            self.sprite_slot_view_mut(k).set_g(value);
        }

        if self.game_state.dungeon.room_tracking.room_index2() == 6
            && self.sprite_slot_view(k).z() == 0
        {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
        }
        // ROM $05:EF79 JSL $06:DBF0
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        self.sprite_move_z(k, SpriteMoveZEntry::Unpriced);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if self.game_state.dungeon.room_tracking.room_index2() == 6
                && self.sprite_slot_view(k).subtype() == 0
            {
                self.sprite_slot_view_mut(k).add_flags2(2);
                let value = 1;
                self.sprite_slot_view_mut(k).set_subtype(value);
                self.sprite_spawn_water_splash(k);
            }
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        // ROM $05:EFBF
        if !self.sprite_check_damage_same_layer_long_marked(k) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        if self.sprite_slot_view(k).a() != 0 {
            self.follower_link_state_mut().set_item_receipt_method(2);
            if self
                .link_receive_item_from(
                    0x3e,
                    0,
                    ItemReceiptCaller::SpriteMainDirect {
                        sprite_slot: k as u8,
                        suffix: SpriteMainItemReceiptSuffix::HeartContainerFull,
                    },
                )
                .is_suspended()
            {
                return;
            }
            self.complete_heart_container_full_item_receipt();
            return;
        }
        self.link_cancel_dash();
        self.follower_link_state_mut().set_item_receipt_method(0);
        if self
            .link_receive_item_from(
                0x26,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::HeartContainerUpgrade,
                },
            )
            .is_suspended()
        {
            return;
        }
        self.complete_heart_container_upgrade_item_receipt(k);
    }

    /// Source suffix after Sprite_HeartContainer's crystal-branch
    /// `Link_ReceiveItem(0x3e)` call (ROM `$85ef47`). A live timing authority
    /// suspends the decompressor there; only the savegame-state bit remains.
    pub(in crate::zelda_rtl) fn complete_heart_container_full_item_receipt(&mut self) {
        let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x8000;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
    }

    /// Source suffix after Sprite_HeartContainer's piece-upgrade
    /// `Link_ReceiveItem(0x26)` call (ROM `$85ef47`).
    pub(in crate::zelda_rtl) fn complete_heart_container_upgrade_item_receipt(&mut self, k: usize) {
        self.heart_upgrade_set_obtained_flag(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HeartPiece(int k) {  // 85f020
    pub(super) fn sprite_heart_piece(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_slot_view_mut(k).add_ai_state(1);
            self.heart_upgrade_check_if_already_obtained(k);
            if self.sprite_slot_view(k).state() == 0 {
                return;
            }
        }
        // ROM $05:F030 JSL $06:DBF0
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }

        // ROM $05:F03D -> $05:B88D -> $06:E496
        if (self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Rom06E496) & 3) != 0 {
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).x_velocity());
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }

        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        self.sprite_move_z(k, SpriteMoveZEntry::Unpriced);
        // ROM $05:F056
        self.sprite_move_xy(k, SpriteMoveEntry::Rom05FA00);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = ((self.sprite_slot_view(k).z_velocity() ^ 255) & 248) >> 1;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = (self.sprite_slot_view(k).x_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }

        if self.sprite_slot_view(k).delay_aux4() != 0
            // ROM $05:F083
            || !self.sprite_check_damage_same_layer_long_marked(k)
        {
            return;
        }

        let heart_pieces = self.player_resources_mut().advance_heart_piece_count();
        if heart_pieces == 0 {
            self.link_cancel_dash();
            self.follower_link_state_mut().set_item_receipt_method(0);
            if self
                .link_receive_item_from(
                    0x26,
                    0,
                    ItemReceiptCaller::SpriteMainDirect {
                        sprite_slot: k as u8,
                        suffix: SpriteMainItemReceiptSuffix::HeartPiece,
                    },
                )
                .is_suspended()
            {
                return;
            }
        } else {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2d);
            self.sprite_show_message_unconditional(HEART_PIECE_MESSAGES[usize::from(heart_pieces)]);
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        self.heart_upgrade_set_obtained_flag(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D8_Heart(int k) {  // 86cec0
    pub(super) fn sprite_d8_heart(&mut self, k: usize) {
        if self.sprite_draw_absorbable_transient(k, true) {
            return;
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_check_absorption_by_player(k);

        if self.sprite_slot_view(k).state() == 0
            && self
                .game_state
                .enhanced_features
                .has(FEATURE_MISC_BUG_FIXES_DRAW)
        {
            return;
        }

        if self.sprite_handle_dragging_by_ancilla(k) {
            return;
        }
        // ROM $06:CECC
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        // ROM $06:CECF
        self.sprite_move_z(k, SpriteMoveZEntry::Rom06E96C);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.sprite_slot_view_mut(k).and_oam_flags(!0x40);
        if !sign8(self.sprite_slot_view(k).x_velocity()) {
            self.sprite_slot_view_mut(k).or_oam_flags(0x40);
        }
        match self.sprite_slot_view(k).ai_state().min(3) {
            0 => {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = 18;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 20;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                } else {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 253;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).direction() & 1);
                    let value = self
                        .sprite_slot_view(k)
                        .x_velocity()
                        .wrapping_add(HEART_REFILL_X_ACCELERATIONS[j] as u8);
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if self.sprite_slot_view(k).x_velocity()
                        == HEART_REFILL_X_VELOCITY_TARGETS[j] as u8
                    {
                        self.sprite_slot_view_mut(k).add_direction(1);
                        let value = 8;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
            }
            _ => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_E3_Fairy(int k) {  // 86cf94
    pub(super) fn sprite_e3_fairy(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.game_state.world.location.is_outdoors() {
                let value = 48;
                self.sprite_slot_view_mut(k).set_object_priority(value);
            }
            if self.sprite_draw_absorbable_transient(k, true) {
                return;
            }
        }
        self.fairy_check_if_touchable(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_aux4() == 0 {
                    if self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Unpriced) {
                        self.sprite_handle_absorption_by_player(k);
                    } else if (self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Unpriced)
                        & PLAYER_DAMAGE_NONZERO_MASK_DRAW)
                        != 0
                    {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.sprite_show_message_unconditional(0xc9);
                        return;
                    }
                }
                if self.sprite_slot_view(k).state() == 0
                    && self
                        .game_state
                        .enhanced_features
                        .has(FEATURE_MISC_BUG_FIXES_DRAW)
                {
                    return;
                }
                if self.sprite_handle_dragging_by_ancilla(k) {
                    return;
                }
                self.faerie_handle_movement(k);
            }
            1 => {
                if self.multiselect_choice().value() == 0 {
                    let j = self.sprite_find_empty_bottle();
                    if j >= 0 {
                        let value = 6;
                        self.inventory_items_mut().set_bottle(j as usize, value);
                        self.hud_refresh_icon();
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_state(value);
                        return;
                    }
                    self.sprite_show_message_unconditional(0xca);
                }
                let value = 48;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_D9_GreenRupee(int k) {  // 86d04a
    pub(super) fn sprite_d9_green_rupee(&mut self, k: usize) {
        self.sprite_draw_ripple_if_in_water(k);
        if self.sprite_draw_absorbable_transient(k, true) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Absorbable_Main(int k) {  // 86d051
    pub(in crate::zelda_rtl) fn sprite_absorbable_main(&mut self, k: usize) {
        let _scope=self.lag_free.then(|| crate::cycle_ledger::routine(0x06_d051));
        let blocks=crate::cycle_ledger::blocks(self.lag_free);
        // $06:D051..D0EC, M8/X8: complete control, excluding its separately
        // executed inactivity/movement/collision/bounce/absorption children.
        blocks.block(0x06d051);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        blocks.block(0x06d054);
        // ROM $06:D054
        self.sprite_move_z(k, SpriteMoveZEntry::Rom06E96C);
        blocks.block(0x06d057);
        // ROM $06:D057
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        if matches!(self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot } | SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot } | SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot }) if slot == k as u8)
        {
            assert_eq!(self.sprite_slot_view(k).delay_aux3(), 0);
            assert!(self.game_state.world.location.is_outdoors());
            assert_eq!(self.game_state.dungeon.room_load.header_collision(), 0);
            assert_eq!(self.sprite_slot_view(k).flags2() & 0x20, 0);
            self.sprite_slot_view_mut(k).set_wall_collision(0);
            if self.sprite_main_cpu_boundary
                == Some(
                    SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot: k as u8 },
                )
            {
                assert_ne!(self.sprite_slot_view(k).y_velocity(), 0);
                self.sprite_vertical_tile_attribute_loaded(k);
                return;
            }

            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot: k as u8 })
            {
                assert_ne!(self.sprite_slot_view(k).y_velocity(), 0);
                return;
            }
            if self.sprite_slot_view(k).y_velocity() != 0 {
                self.sprite_check_for_tile_in_direction_vertical(
                    k,
                    if sign8(self.sprite_slot_view(k).y_velocity()) {
                        0
                    } else {
                        1
                    },
                );
            }
            assert_ne!(self.sprite_slot_view(k).x_velocity(), 0);
            return;
        }
        blocks.block(0x06d05a);
        if self.sprite_slot_view(k).delay_aux3() == 0 {
            blocks.block(0x06d05f);
            self.sprite_check_tile_collision2(k);
            blocks.block(0x06d062);
            self.sprite_bounce_off_wall(k);
        } else { blocks.taken(0x06d05d); }
        self.sprite_absorbable_after_collision(k);
    }

    pub(in crate::zelda_rtl) fn sprite_absorbable_after_vertical_lookup(&mut self, k: usize) {
        self.sprite_check_for_tile_in_direction_vertical(
            k,
            if sign8(self.sprite_slot_view(k).y_velocity()) {
                0
            } else {
                1
            },
        );
        self.sprite_absorbable_after_vertical_collision(k);
    }

    pub(in crate::zelda_rtl) fn sprite_absorbable_after_vertical_attribute_loaded(&mut self, k: usize) {
        self.sprite_vertical_collision_after_attribute_loaded(k);
        self.sprite_absorbable_after_vertical_collision(k);
    }

    fn sprite_absorbable_after_vertical_collision(&mut self, k: usize) {
        if self.sprite_slot_view(k).x_velocity() != 0 {
            self.sprite_absorbable_after_horizontal_lookup(k);
        } else {
            self.sprite_check_tile_collision_after_directions(k);
            self.sprite_bounce_off_wall(k);
            self.sprite_absorbable_after_collision(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_absorbable_after_horizontal_lookup(&mut self, k: usize) {
        self.sprite_check_for_tile_in_direction_horizontal(
            k,
            if sign8(self.sprite_slot_view(k).x_velocity()) {
                2
            } else {
                3
            },
        );
        self.sprite_check_tile_collision_after_directions(k);
        self.sprite_bounce_off_wall(k);
        self.sprite_absorbable_after_collision(k);
    }

    fn sprite_absorbable_after_collision(&mut self, k: usize) {
        let blocks=crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x06d065);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            blocks.block(0x06d073);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = (self.sprite_slot_view(k).x_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = (self.sprite_slot_view(k).y_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let t = (0u8).wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 1;
            if t < 9 {
                blocks.block(0x06d08f); blocks.block(0x06d092);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                blocks.taken(0x06d08d); blocks.block(0x06d094);
                let value = t;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let tile=self.sprite_slot_view(k).draw_i();
                if tile==8 { blocks.taken(0x06d09d); }
                else {
                    blocks.block(0x06d09f);
                    if tile!=9 { blocks.taken(0x06d0a1); }
                }
                if tile == 8 || tile == 9 {
                    blocks.block(0x06d0a3);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let j = self.sprite_spawn_small_splash(k);
                    blocks.block(0x06d0aa);
                    if j<0 { blocks.taken(0x06d0aa); }
                    else {
                        blocks.block(0x06d0ac);
                        if self.sprite_slot_view(k).flags3()&0x20==0 { blocks.taken(0x06d0b1); }
                        else { blocks.block(0x06d0b3); }
                    }
                    if j >= 0 && (self.sprite_slot_view(k).flags3() & 0x20) != 0 {
                        let j = j as usize;
                        // The ROM subtracts with SBC and no SEC (the C port's
                        // "wtf carry propagation"): the carry is whatever
                        // Sprite_CalculateSfxPan's final compare left inside
                        // SpriteSfx_QueueSfx2WithPan — clear for a centered
                        // pan, set otherwise — and the y subtraction chains
                        // from the x borrow (route host 450016: the splash
                        // lands at x-5/y-4 where the C port placed x-4/y-4).
                        let carry_in = self.sprite_calculate_sfx_pan(k) != 0;
                        let (x, carry) = crate::types::sbc_u16(self.sprite_get_x(j), 4, carry_in);
                        self.sprite_set_x(j, x);
                        let (y, _) = crate::types::sbc_u16(self.sprite_get_y(j), 4, carry);
                        self.sprite_set_y(j, y);
                    }
                } else {
                    blocks.block(0x06d0d5);
                    if self.sprite_slot_view(k).sprite_type()<0xe4 { blocks.taken(0x06d0da); }
                    else {
                        blocks.block(0x06d0dc);
                        if self.game_state.world.location.is_outdoors() { blocks.taken(0x06d0de); }
                        else {
                            blocks.block(0x06d0e0);
                            self.sprite_sfx_queue_sfx2_with_pan(k,5);
                        }
                    }
                }
            }
        }
        else { blocks.taken(0x06d071); }
        blocks.block(0x06d0e6);
        if self.sprite_handle_dragging_by_ancilla(k) {
            return;
        }
        blocks.block(0x06d0e9);
        self.sprite_check_absorption_by_player(k);
        blocks.block(0x06d0ec);
    }

    // -----------------------------------------------------------------------
    // void DiggingGameGuy_Draw(int k) {  // 9dfe4b
    pub(in crate::zelda_rtl) fn digging_game_guy_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &DIGGING_GAME_GUY_DRAW_FRAMES[base..base + 3],
        );
        // ROM $1D:FE69
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void Bot_Draw(int k) {  // 85b89a
    pub(super) fn bot_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).a());
        let value = BOT_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | BOT_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:B8AE JSL $06:DBF0
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
    }

    // -----------------------------------------------------------------------
    // void BombShopEntity_Draw(int k) {  // 9ee2c6
    pub(super) fn bomb_shop_entity_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).subtype2()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &BOMB_SHOP_ENTITY_DRAW_FRAMES[base..base + 1],
        );
        // ROM $1E:E2E4
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B3_PedestalPlaque(int k) {  // 9ee044
    pub(super) fn sprite_b3_pedestal_plaque(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if !self.game_state.player.follower_link.is_immobilized()
            && self.sprite_check_if_link_is_busy()
        {
            return;
        }

        self.follower_link_state_mut()
            .clear_position_mode_bits(0x20);
        if self.game_state.world.location.overworld_screen_index() != 48 {
            if self.game_state.player.follower_link.has_facing()
                // ROM $1E:E069, $1E:E0A3
                || !self.sprite_check_damage_same_layer_long_marked(k)
            {
                return;
            }
            if self.game_state.inventory.save_progress.hud_current_item()
                != HUD_ITEM_BOOK_MUDORA_DRAW
                || (self.game_state.player.follower_link.filtered_joypad_h() & 0x40) == 0
            {
                if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0 {
                    return;
                }
                self.sprite_show_message_unconditional(0x00b6);
            } else {
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut().set_position_mode(32);
                self.set_sound_effect_1(0);
                self.sprite_show_message_unconditional(0x00b7);
            }
        } else {
            if self.game_state.player.follower_link.has_facing()
                // ROM $1E:E069, $1E:E0A3
                || !self.sprite_check_damage_same_layer_long_marked(k)
            {
                return;
            }
            if self.game_state.inventory.save_progress.hud_current_item()
                != HUD_ITEM_BOOK_MUDORA_DRAW
                || (self.game_state.player.follower_link.filtered_joypad_h() & 0x40) == 0
            {
                if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0 {
                    return;
                }
                self.sprite_show_message_unconditional(0x00bc);
            } else {
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut().set_position_mode(32);
                self.set_sound_effect_1(0);
                self.follower_link_state_mut().set_button_b_frames(1);
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
                self.follower_link_state_mut()
                    .set_handler_state(PLAYER_STATE_OPENING_DESERT_PALACE_DRAW);
                self.sprite_show_message_unconditional(0x00bd);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B4_PurpleChest(int k) {  // 9ee0dd
    pub(super) fn sprite_b4_purple_chest(&mut self, k: usize) {
        if self.sprite_b4_purple_chest_before_follower_graphics(k) {
            self.load_follower_graphics();
            self.sprite_become_follower(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_b4_purple_chest_before_follower_graphics(&mut self, k: usize) -> bool {
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return false;
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.sprite_show_message_on_contact(k, 0x0116) & 0x100 != 0
                && self.game_state.sprites.follower_runtime.indicator() == 0
            {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_state_mut().set_indicator(12);
            return true;
        }
        false
    }

    // -----------------------------------------------------------------------
    // void Sprite_B5_BombShop(int k) {  // 9ee111
    pub(super) fn sprite_b5_bomb_shop(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_bomb_shop_clerk(k),
            1 => self.sprite_bomb_shop_bomb(k),
            2 => self.sprite_bomb_shop_super_bomb(k),
            3 => self.sprite_bomb_shop_huff(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Clerk(int k) {  // 9ee134
    pub(super) fn sprite_bomb_shop_clerk(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let j = usize::from(self.sprite_slot_view(k).e());
            let value = self.sprite_slot_view(k).e().wrapping_add(1) & 7;
            self.sprite_slot_view_mut(k).set_e(value);
            let value = SPRITE_BOMB_SHOP_CLERK_DELAY[j];
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = SPRITE_BOMB_SHOP_CLERK_LOCAL_GRAPHICS[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).graphics() == 0 {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x11);
                self.bomb_shop_clerk_exhalation(k);
            } else {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x12);
            }
        }
        let flag = (self.game_state.inventory.player_resources.crystal_flags() & 5) == 5
            && (self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 32)
                != 0;
        self.sprite_show_solicited_message(k, if flag { 0x0118 } else { 0x0117 });
        self.sprite_behave_as_barrier(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Bomb(int k) {  // 9ee190
    pub(super) fn sprite_bomb_shop_bomb(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if !self.shop_item_check_for_a_press(k) {
            return;
        }

        let resources = &self.game_state.inventory.player_resources;
        let upgrade = resources.bomb_upgrade_level() as usize;
        if resources.bombs() != MAX_BOMBS_FOR_LEVEL_DRAW[upgrade] {
            if !self.shop_item_handle_cost(100) {
                self.sprite_show_message_unconditional(0x017c);
                self.shop_item_play_beep(k);
            } else {
                self.player_resources_mut().set_bomb_filler(27);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_show_message_unconditional(0x0119);
                self.shop_item_handle_receipt(k, 0x28);
            }
        } else {
            self.sprite_show_message_unconditional(0x016e);
            self.shop_item_play_beep(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_SuperBomb(int k) {  // 9ee1df
    pub(in crate::zelda_rtl) fn sprite_bomb_shop_super_bomb(&mut self, k: usize) {
        if self.sprite_bomb_shop_super_bomb_before_follower_graphics(k) {
            self.load_follower_graphics();
            self.sprite_bomb_shop_super_bomb_after_follower_graphics(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_bomb_shop_super_bomb_before_follower_graphics(
        &mut self,
        k: usize,
    ) -> bool {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return false;
        }
        self.sprite_behave_as_barrier(k);
        if self.shop_item_check_for_a_press(k) {
            if !self.shop_item_handle_cost(100) {
                self.sprite_show_message_unconditional(0x017c);
                self.shop_item_play_beep(k);
            } else {
                self.follower_state_mut().set_indicator(13);
                return true;
            }
        }
        false
    }

    pub(in crate::zelda_rtl) fn sprite_bomb_shop_super_bomb_after_follower_graphics(&mut self, k: usize) {
        self.sprite_become_follower(k);
        self.sprite_slot_view_mut(k).set_state(0);
        self.sprite_show_message_unconditional(0x011a);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Huff(int k) {  // 9ee21a
    pub(super) fn sprite_bomb_shop_huff(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        // ROM $1E:E220
        self.sprite_draw_single_small(k, SpriteSingleSmallEntry::Rom06DBF8);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).and_oam_flags(0x30);
        let value = SPRITE_BOMB_SHOP_HUFF_OAM_FLAGS
            [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
        self.sprite_slot_view_mut(k).or_oam_flags(value);
        self.sprite_slot_view_mut(k).add_z_velocity(1);
        // ROM $1E:E242
        self.sprite_move_z(k, SpriteMoveZEntry::Rom1EFF62);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let value = self.sprite_slot_view(k).delay_main() >> 3 & 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BB_Shopkeeper(int k) {  // 9eeeef
    pub(super) fn sprite_bb_shopkeeper(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.shopkeeper_standard_clerk(k),
            1 => self.chest_game_guy(k),
            2 => self.nice_thief_with_gift(k),
            3 => self.mini_chest_game_guy(k),
            4 => self.lost_woods_chest_game_guy(k),
            5 | 6 => self.nice_thief_under_rock(k),
            7 => self.shop_item_red_potion150(k),
            8 => self.shop_item_fighter_shield(k),
            9 => self.shop_item_fire_shield(k),
            10 => self.shop_item_heart(k),
            11 => self.shop_item_arrows(k),
            12 => self.shop_item_bombs(k),
            13 => self.shop_item_bee(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Shopkeeper_StandardClerk(int k) {  // 9eef12
    pub(super) fn shopkeeper_standard_clerk(&mut self, k: usize) {
        if self.game_state.world.region.is_in_dark_world() {
            self.oam_allocate_defer_to_player(k);
            // ROM $1E:EF1B JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
            if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                return;
            }
            let value = (self.sprite_slot_view(k).oam_flags() & 63)
                | ((self.game_state.frame.frame_counter << 3) & 64);
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let value = 7;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            self.shopkeeper_draw(k);
            if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                return;
            }
            let value = (self.game_state.frame.frame_counter >> 4) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.sprite_behave_as_barrier(k);
        let msg = if !self.game_state.world.region.is_in_dark_world() {
            0x0165
        } else {
            0x015f
        };
        self.sprite_show_solicited_message(k, msg);
        if self.sprite_slot_view(k).ai_state() == 0
            && self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(0x60)
                >= self.game_state.player.follower_link.y()
        {
            self.sprite_show_message_unconditional(msg);
            let value = 1;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void ChestGameGuy(int k) {  // 9eef90
    pub(super) fn chest_game_guy(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        // ROM $1E:EF94 JSL $06:DBF0
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        let value = (self.sprite_slot_view(k).oam_flags() & 63)
            | ((self.game_state.frame.frame_counter << 3) & 64);
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x0160) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(30) {
                    self.minigame_state_mut().set_credits(2);
                    self.sprite_show_message_unconditional(0x0164);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0161);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                if self.game_state.minigame.credits() == 0 {
                    self.sprite_show_solicited_message(k, 0x0163);
                } else {
                    self.sprite_show_solicited_message(k, 0x017f);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void NiceThiefWithGift(int k) {  // 9ef038
    pub(super) fn nice_thief_with_gift(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_show_solicited_message(k, 0x0176) & 0x100 != 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x4000 == 0 {
                    let bits =
                        self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x4000;
                    self.dungeon_savegame_state_mut()
                        .set_savegame_state_bits(bits);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.shop_item_handle_receipt(k, 0x46);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MiniChestGameGuy(int k) {  // 9ef078
    pub(super) fn mini_chest_game_guy(&mut self, k: usize) {
        let value = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.maze_game_guy_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x017e) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(20) {
                    self.minigame_state_mut().set_credits(1);
                    self.sprite_show_message_unconditional(0x017f);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0180);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_show_solicited_message(
                    k,
                    if self.game_state.minigame.credits() == 0 {
                        0x0163
                    } else {
                        0x017f
                    },
                );
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void LostWoodsChestGameGuy(int k) {  // 9ef0f3
    pub(super) fn lost_woods_chest_game_guy(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x0181) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(100) {
                    self.minigame_state_mut().set_credits(1);
                    self.sprite_show_message_unconditional(0x017f);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0180);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_show_solicited_message(
                    k,
                    if self.game_state.minigame.credits() == 0 {
                        0x0163
                    } else {
                        0x017f
                    },
                );
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void NiceThiefUnderRock(int k) {  // 9ef14f
    pub(super) fn nice_thief_under_rock(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(
            k,
            if self.sprite_slot_view(k).subtype2() == 5 {
                0x0177
            } else {
                0x0178
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_BC_Drunkard(int k) {  // 9ef603
    pub(super) fn sprite_bc_drunkard(&mut self, k: usize) {
        self.drinking_guy_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.get_random_number() == 0 {
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        let value = u8::from(self.sprite_slot_view(k).delay_main() != 0);
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_show_solicited_message(k, 0x0175) & 0x100 != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void BigFaerie_Draw(int k) {  // 9dc5d0
    pub(super) fn big_faerie_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut info = self.sprite_draw_multiple(k, &BIG_FAERIE_DRAW_FRAMES[base..base + 4]);
        // ROM $1D:C5E9
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void FaerieCloud_Draw(int k) {  // 9dc616
    pub(super) fn faerie_cloud_draw(&mut self, k: usize) {
        if !sign8(self.sprite_slot_view(k).a())
            && (self.sprite_slot_view(k).a() & self.sprite_slot_view(k).subtype2()) == 0
        {
            let x = FAERIE_CLOUD_DRAW_XY[usize::from(self.get_random_number() & 7)];
            let y = FAERIE_CLOUD_DRAW_XY[usize::from(self.get_random_number() & 7)];
            self.sprite_garnish_spawn_sparkle(k, x, y);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_C8_BigFairy(int k) {  // 9dc414
    pub(super) fn sprite_c8_big_fairy(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            self.sprite_fairy_cloud(k);
        } else {
            self.sprite_big_fairy(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_FairyCloud(int k) {  // 9dc41c
    pub(super) fn sprite_fairy_cloud(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.faerie_cloud_draw(k);
        if (self.sprite_slot_view(k).subtype2() & 31) == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x31);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_a(value);
                self.sprite_apply_speed_towards_link(k, 8);
                self.sprite_move_xy(k, SpriteMoveEntry::Rom1DE95D);
                self.sprite_get16_bit_coords(k);
                let x = self
                    .game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                    .wrapping_add(3);
                let y = self
                    .game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                    .wrapping_add(11);
                if x < 6 && y < 6 {
                    self.player_resources_mut()
                        .increment_heart_filler_word_by(0x00a0);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let resources = &self.game_state.inventory.player_resources;
                if resources.current_health() == resources.health_capacity() {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.sprite_slot_view_mut(0).set_delay_aux2(112);
                }
            }
            2 => {
                if (self.sprite_slot_view(k).subtype2() & 15) != 0
                    || sign8(self.sprite_slot_view(k).a())
                {
                    return;
                }
                let value = self.sprite_slot_view(k).a().wrapping_mul(2).wrapping_add(1);
                self.sprite_slot_view_mut(k).set_a(value);
                if self.sprite_slot_view(k).a() >= 0x80 {
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_a(value);
                    self.follower_link_state_mut().clear_immobilized();
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BigFairy(int k) {  // 9dc4bf
    pub(super) fn sprite_big_fairy(&mut self, k: usize) {
        let mut i = self.sprite_slot_view(k).delay_aux2();
        if i != 0 && i < 0x40 {
            i = i.wrapping_sub(1);
            if i == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if (i & 1) != 0 {
                return;
            }
        }
        self.big_faerie_draw(k);
        self.sprite_slot_view_mut(k).subtract_g(1);
        if sign8(self.sprite_slot_view(k).g()) {
            let value = 5;
            self.sprite_slot_view_mut(k).set_g(value);
            let value = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.faerie_cloud_draw(k);
                let value = 1;
                self.sprite_slot_view_mut(k).set_a(value);
                let (_, pt) = self.sprite_direction_and_offset_to_face_link(k);
                if pt.x.wrapping_add(0x30) < 0x60 && pt.y.wrapping_add(0x30) < 0x60 {
                    self.link_cancel_dash();
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.dialogue_message_index_mut().set_value(0x015a);
                    self.sprite_show_message_minimal_c();
                    self.follower_link_state_mut().immobilize();
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xc8, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_BigFairy expected Sprite_SpawnDynamically to succeed"
                    );
                    let j = j as usize;
                    self.sprite_set_spawned_coordinates(j, &info);
                    let value = 1;
                    self.sprite_slot_view_mut(j).set_head_direction(value);
                    let value = self
                        .sprite_slot_view(j)
                        .y_low()
                        .wrapping_sub(self.sprite_slot_view(k).z());
                    self.sprite_slot_view_mut(j).set_y_low(value);
                    let value = 0;
                    self.sprite_slot_view_mut(j).set_z(value);
                }
            }
            1 => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_AC_Apple(int k) {  // 9ef515
    pub(super) fn sprite_ac_apple(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_apple(k);
            return;
        }
        if self.sprite_slot_view(k).e() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            let mut n = (self.get_random_number() & 3).wrapping_add(2);
            loop {
                self.spawn_apple(k);
                if n == 0 {
                    break;
                }
                n = n.wrapping_sub(1);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Apple(int k) {  // 9ef57c
    pub(super) fn sprite_apple(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() >= 16 || (self.game_state.frame.frame_counter & 2) != 0 {
            // ROM $1E:F589 JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_slot_view(k).a() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        self.sprite_move_xyz(k, SpriteMoveEntry::Rom1EFF34);
        // $1E:F598 JSR $1E:C26B: complete body, caller opcode unchanged.
        if self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Rom1EC26B) {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x0b);
            self.player_resources_mut().increment_heart_filler_by(8);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        if (self.game_state.frame.frame_counter & 1) == 0 {
            self.sprite_slot_view_mut(k).subtract_a(1);
        }

        if !sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
            self.sprite_slot_view_mut(k).subtract_z_velocity(1);
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_z(value);
        let a = if sign8(self.sprite_slot_view(k).z_velocity()) {
            self.sprite_slot_view(k).z_velocity()
        } else {
            0
        };
        let value = a.wrapping_neg() >> 1;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        if self.sprite_slot_view(k).x_velocity() != 0 {
            let value = self.sprite_slot_view(k).x_velocity().wrapping_add(
                if sign8(self.sprite_slot_view(k).x_velocity()) {
                    1
                } else {
                    0xff
                },
            );
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        if self.sprite_slot_view(k).y_velocity() != 0 {
            let value = self.sprite_slot_view(k).y_velocity().wrapping_add(
                if sign8(self.sprite_slot_view(k).y_velocity()) {
                    1
                } else {
                    0xff
                },
            );
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }
    }

    pub(in crate::zelda_rtl) fn archery_game_draw_prize(&mut self, k: usize) {
        let Some((info_x, info_y, _info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet)
        else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize() + 4;
        let b = usize::from(self.sprite_slot_view(k).b()).wrapping_sub(1);
        for i in (0..5).rev() {
            let charnum = if i == 4 {
                ARCHERY_GAME_DRAW_PRIZE_CHARS_4[b]
            } else if i == 3 {
                ARCHERY_GAME_DRAW_PRIZE_CHARS_3[b]
            } else {
                ARCHERY_GAME_DRAW_PRIZE_CHARS[i]
            };
            let flags = ARCHERY_GAME_DRAW_PRIZE_FLAGS[i] & if charnum < 0x7c { 0xff } else { 0xfe };
            self.set_oam_plain_at(
                oam,
                info_x.wrapping_add(ARCHERY_GAME_DRAW_PRIZE_X_OFFSETS[i] as i16 as u16) as u8,
                info_y.wrapping_add(ARCHERY_GAME_DRAW_PRIZE_Y_OFFSETS[i] as i16 as u16) as u8,
                charnum,
                flags,
                0,
            );
            oam += 4;
        }
        self.sprite_draw_distress_custom(info_x, info_y, self.game_state.frame.frame_counter);
    }

    pub(in crate::zelda_rtl) fn archery_game_guy_draw(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        let mut info = self.sprite_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let j = g * 3 + i;
            self.set_oam_plain_at(
                oam,
                info.x
                    .wrapping_add(ARCHERY_GAME_GUY_DRAW_X_OFFSETS[j] as i16 as u16)
                    as u8,
                info.y
                    .wrapping_add(ARCHERY_GAME_GUY_DRAW_Y_OFFSETS[j] as i16 as u16)
                    as u8,
                ARCHERY_GAME_GUY_DRAW_CHARS[j],
                ARCHERY_GAME_GUY_DRAW_FLAGS[j] | info.flags,
                ARCHERY_GAME_GUY_DRAW_BIG[j],
            );
            oam += 4;
        }
        // ROM $0D:DB11
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }
}
