//! Semantic inventory effects of item receipts. Animation and suspension stay
//! in the receipt caller; effects are applied at its original observation points.
use super::*;
use crate::game_state::EquipmentItem;

pub(crate) fn chest_item_alternate(item: u8) -> Option<(EquipmentItem, u8)> {
    match item {
        0x0c => Some((EquipmentItem::Boomerang, 0x44)),
        0x12 => Some((EquipmentItem::Torch, 0x35)),
        0x2a => Some((EquipmentItem::Boomerang, 0x46)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
enum InitialItemAward {
    Equipment(EquipmentItem, u8),
    ArrowRefill(u8),
}

fn initial_item_award(item: u8) -> Option<InitialItemAward> {
    match item {
        0x00 => Some(InitialItemAward::Equipment(EquipmentItem::Sword, 1)),
        0x01 => Some(InitialItemAward::Equipment(EquipmentItem::Sword, 2)),
        0x02 => Some(InitialItemAward::Equipment(EquipmentItem::Sword, 3)),
        0x03 => Some(InitialItemAward::Equipment(EquipmentItem::Sword, 4)),
        0x04 => Some(InitialItemAward::Equipment(EquipmentItem::Shield, 1)),
        0x05 => Some(InitialItemAward::Equipment(EquipmentItem::Shield, 2)),
        0x06 => Some(InitialItemAward::Equipment(EquipmentItem::Shield, 3)),
        0x07 => Some(InitialItemAward::Equipment(EquipmentItem::FireRod, 1)),
        0x08 => Some(InitialItemAward::Equipment(EquipmentItem::IceRod, 1)),
        0x09 => Some(InitialItemAward::Equipment(EquipmentItem::Hammer, 1)),
        0x0a => Some(InitialItemAward::Equipment(EquipmentItem::Hookshot, 1)),
        0x0b => Some(InitialItemAward::Equipment(EquipmentItem::Bow, 1)),
        0x0c => Some(InitialItemAward::Equipment(EquipmentItem::Boomerang, 1)),
        0x0d => Some(InitialItemAward::Equipment(EquipmentItem::Mushroom, 2)),
        0x0f => Some(InitialItemAward::Equipment(EquipmentItem::Bombos, 1)),
        0x10 => Some(InitialItemAward::Equipment(EquipmentItem::Ether, 1)),
        0x11 => Some(InitialItemAward::Equipment(EquipmentItem::Quake, 1)),
        0x12 => Some(InitialItemAward::Equipment(EquipmentItem::Torch, 1)),
        0x13 => Some(InitialItemAward::Equipment(EquipmentItem::Flute, 1)),
        0x14 => Some(InitialItemAward::Equipment(EquipmentItem::Flute, 2)),
        0x15 => Some(InitialItemAward::Equipment(EquipmentItem::CaneSomaria, 1)),
        0x18 => Some(InitialItemAward::Equipment(EquipmentItem::CaneByrna, 1)),
        0x19 => Some(InitialItemAward::Equipment(EquipmentItem::Cape, 1)),
        0x1a => Some(InitialItemAward::Equipment(EquipmentItem::Mirror, 2)),
        0x1b => Some(InitialItemAward::Equipment(EquipmentItem::Gloves, 1)),
        0x1c => Some(InitialItemAward::Equipment(EquipmentItem::Gloves, 2)),
        0x1d => Some(InitialItemAward::Equipment(EquipmentItem::Book, 1)),
        0x1e => Some(InitialItemAward::Equipment(EquipmentItem::Flippers, 1)),
        0x1f => Some(InitialItemAward::Equipment(EquipmentItem::MoonPearl, 1)),
        0x21 => Some(InitialItemAward::Equipment(EquipmentItem::BugNet, 1)),
        0x23 => Some(InitialItemAward::Equipment(EquipmentItem::Armor, 2)),
        0x2a => Some(InitialItemAward::Equipment(EquipmentItem::Boomerang, 2)),
        0x3a => Some(InitialItemAward::Equipment(EquipmentItem::Bow, 1)),
        0x3b => Some(InitialItemAward::Equipment(EquipmentItem::Bow, 3)),
        0x43 => Some(InitialItemAward::ArrowRefill(1)),
        0x44 => Some(InitialItemAward::ArrowRefill(10)),
        0x49 => Some(InitialItemAward::Equipment(EquipmentItem::Sword, 1)),
        0x4a => Some(InitialItemAward::Equipment(EquipmentItem::Flute, 3)),
        0x4b => Some(InitialItemAward::Equipment(EquipmentItem::Boots, 1)),
        _ => None,
    }
}

impl ZeldaState {
    pub(super) fn grant_initial_item_award(&mut self, item: u8) {
        // The first sword also grants the first shield, before the sword write.
        if item == 0 {
            self.inventory_items_mut()
                .grant_equipment(EquipmentItem::Shield, 1);
        }
        match initial_item_award(item) {
            Some(InitialItemAward::Equipment(kind, value)) => {
                self.inventory_items_mut().grant_equipment(kind, value);
            }
            Some(InitialItemAward::ArrowRefill(value)) => {
                self.player_resources_mut().set_arrow_filler(value);
            }
            None => {}
        }
    }
}
