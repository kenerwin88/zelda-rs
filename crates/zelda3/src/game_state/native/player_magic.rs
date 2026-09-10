use super::ram_byte;
use crate::game_state::constants::{LINK_MAGIC_CONSUMPTION, LINK_MAGIC_FILLER, LINK_MAGIC_POWER};
use crate::game_state::native::ram_target::RamTarget;

/// Native magic ownership. The cartridge layout is confined to the import and
/// publication methods; arithmetic deliberately retains the original byte rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerMagicState {
    amount: u8,
    refill: u8,
    consumption_level: u8,
}

impl PlayerMagicState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            amount: ram_byte(ram, LINK_MAGIC_POWER),
            refill: ram_byte(ram, LINK_MAGIC_FILLER),
            consumption_level: ram_byte(ram, LINK_MAGIC_CONSUMPTION),
        }
    }

    pub(crate) fn amount(&self) -> u8 {
        self.amount
    }
    pub(crate) fn refill(&self) -> u8 {
        self.refill
    }
    pub(crate) fn consumption_level(&self) -> u8 {
        self.consumption_level
    }
    pub(super) fn set_refill(&mut self, value: u8) {
        self.refill = value;
    }
    pub(super) fn add_refill(&mut self, value: u8) {
        self.refill = self.refill.wrapping_add(value);
    }
    pub(super) fn decrement_refill(&mut self) {
        self.refill = self.refill.wrapping_sub(1);
    }
    pub(super) fn set_consumption_level(&mut self, value: u8) {
        self.consumption_level = value;
    }

    // These distinct observation/publication boundaries preserve legacy save
    // transfers while consumers are migrated. Resource edits must not re-stamp
    // the amount: its publication historically belongs to player operations.
    pub(crate) fn import_amount(&mut self, ram: &[u8]) {
        self.amount = ram_byte(ram, LINK_MAGIC_POWER);
    }
    pub(crate) fn import_consumption_level(&mut self, ram: &[u8]) {
        self.consumption_level = ram_byte(ram, LINK_MAGIC_CONSUMPTION);
    }
    pub(super) fn publish_amount<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(LINK_MAGIC_POWER, self.amount);
    }
    pub(super) fn publish_resource_fields<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(LINK_MAGIC_CONSUMPTION, self.consumption_level);
        ram.write_byte(LINK_MAGIC_FILLER, self.refill);
    }

    fn spend(&mut self, cost: u8) -> bool {
        let remaining = self.amount.wrapping_sub(cost);
        if self.amount != 0 && remaining < 0x80 {
            self.amount = remaining;
            true
        } else {
            false
        }
    }

    fn refund(&mut self, cost: u8, clamp_full: bool) {
        let amount = u16::from(self.amount) + u16::from(cost);
        self.amount = if clamp_full { amount.min(128) } else { amount } as u8;
    }
}

pub(crate) struct NativePlayerMagicBridgeMut<'a> {
    magic: &'a mut PlayerMagicState,
    ram: &'a mut [u8],
}

impl<'a> NativePlayerMagicBridgeMut<'a> {
    pub(crate) fn new(magic: &'a mut PlayerMagicState, ram: &'a mut [u8]) -> Self {
        magic.import_amount(ram);
        Self { magic, ram }
    }
    pub(crate) fn spend_magic(&mut self, cost: u8) -> bool {
        let spent = self.magic.spend(cost);
        if spent {
            self.magic.publish_amount(self.ram);
        }
        spent
    }
    pub(crate) fn refund_magic(&mut self, cost: u8, clamp_full: bool) {
        self.magic.refund(cost, clamp_full);
        self.magic.publish_amount(self.ram);
    }
    pub(crate) fn decrement_magic_power(&mut self) -> u8 {
        self.magic.amount = self.magic.amount.wrapping_sub(1);
        self.magic.publish_amount(self.ram);
        self.magic.amount
    }
    pub(crate) fn set_magic_power(&mut self, value: u8) {
        self.magic.amount = value;
        self.magic.publish_amount(self.ram);
    }
    pub(crate) fn increment_magic_power(&mut self) {
        self.magic.amount = self.magic.amount.wrapping_add(1);
        self.magic.publish_amount(self.ram);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_publication_is_scoped_and_snapshots_are_independent() {
        let mut ram = vec![0xa5; 0x20000];
        ram[LINK_MAGIC_POWER] = 41;
        ram[LINK_MAGIC_FILLER] = 7;
        ram[LINK_MAGIC_CONSUMPTION] = 2;
        let mut magic = PlayerMagicState::load_from_ram(&ram);
        let snapshot = bincode::serialize(&magic).unwrap();
        assert_eq!(snapshot.len(), 3);
        let saved: PlayerMagicState = bincode::deserialize(&snapshot).unwrap();
        let mut expected = ram.clone();
        {
            let mut bridge = NativePlayerMagicBridgeMut::new(&mut magic, &mut ram);
            assert!(bridge.spend_magic(9));
            bridge.refund_magic(3, false);
        }
        expected[LINK_MAGIC_POWER] = 35;
        assert_eq!(ram, expected);
        assert_eq!(saved.amount(), 41);
        assert_eq!((saved.refill(), saved.consumption_level()), (7, 2));

        // Resource publication must not overwrite an amount updated by a legacy
        // save transfer between import boundaries.
        ram[LINK_MAGIC_POWER] = 66;
        magic.set_refill(0x80);
        magic.set_consumption_level(1);
        magic.publish_resource_fields(&mut ram);
        expected[LINK_MAGIC_POWER] = 66;
        expected[LINK_MAGIC_FILLER] = 0x80;
        expected[LINK_MAGIC_CONSUMPTION] = 1;
        assert_eq!(ram, expected);
        magic.import_amount(&ram);
        assert_eq!(magic.amount(), 66);
    }

    #[test]
    fn spending_and_refunds_preserve_all_byte_inputs() {
        for amount in 0..=255u8 {
            for cost in 0..=255u8 {
                let mut magic = PlayerMagicState {
                    amount,
                    refill: 17,
                    consumption_level: 2,
                };
                let remaining = amount.wrapping_sub(cost);
                let succeeds = amount != 0 && remaining & 0x80 == 0;
                assert_eq!(magic.spend(cost), succeeds);
                assert_eq!(magic.amount(), if succeeds { remaining } else { amount });
                assert_eq!((magic.refill(), magic.consumption_level()), (17, 2));
                for clamp in [false, true] {
                    magic.amount = amount;
                    magic.refund(cost, clamp);
                    let sum = u16::from(amount) + u16::from(cost);
                    assert_eq!(
                        magic.amount(),
                        if clamp && sum >= 128 { 128 } else { sum as u8 }
                    );
                }
            }
        }
    }
}
