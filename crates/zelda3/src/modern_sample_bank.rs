//! Modern-owned BRR instruments and echo seeds.
//!
//! The checked-in assets are exported once from the reference uploads. Normal
//! playback consumes this typed catalog directly; raw SPC RAM remains an
//! oracle-only compatibility input.

use std::sync::OnceLock;

#[derive(serde::Deserialize)]
struct PackedSampleBank {
    sample_rate: u32,
    samples: Vec<PackedSample>,
    banks: Vec<PackedBank>,
}
#[derive(serde::Deserialize)]
struct PackedSample {
    brr: Vec<u8>,
}
#[derive(serde::Deserialize)]
struct PackedBank {
    id: u8,
    name: String,
    instruments: Vec<PackedInstrument>,
    echo_start: usize,
    echo_seed: Vec<u8>,
}
#[derive(serde::Deserialize)]
struct PackedInstrument {
    source: u8,
    sample_index: usize,
    loop_offset: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct ModernSampleRef {
    pub brr: &'static [u8],
    pub loop_offset: usize,
}

fn catalog() -> &'static PackedSampleBank {
    static CATALOG: OnceLock<PackedSampleBank> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let catalog: PackedSampleBank = bincode::deserialize(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/modern_sample_bank.bin"
        )))
        .expect("compiled modern sample bank");
        assert_eq!(catalog.sample_rate, 32_000);
        catalog
    })
}

fn bank(id: u8) -> &'static PackedBank {
    catalog()
        .banks
        .iter()
        .find(|bank| bank.id == id)
        .unwrap_or_else(|| panic!("unknown modern sample bank {id}"))
}

pub(crate) fn sample(id: u8, source: u8) -> Option<ModernSampleRef> {
    let catalog = catalog();
    let instrument = bank(id)
        .instruments
        .iter()
        .find(|instrument| instrument.source == source)?;
    let sample = catalog.samples.get(instrument.sample_index)?;
    Some(ModernSampleRef {
        brr: &sample.brr,
        loop_offset: instrument.loop_offset,
    })
}

pub(crate) fn echo_bytes(id: u8, address: usize) -> Option<[u8; 4]> {
    let bank = bank(id);
    let offset = address.checked_sub(bank.echo_start)?;
    Some(bank.echo_seed.get(offset..offset + 4)?.try_into().unwrap())
}

pub(crate) fn bank_name(id: u8) -> &'static str {
    &bank(id).name
}

pub(crate) fn is_valid_bank(id: u8) -> bool {
    catalog().banks.iter().any(|bank| bank.id == id)
}

pub(crate) fn identify_spc_ram(ram: &[u8]) -> Option<u8> {
    catalog().banks.iter().find_map(|bank| {
        (ram.get(bank.echo_start..) == Some(bank.echo_seed.as_slice())).then_some(bank.id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_three_complete_banks_and_deduplicated_aliases() {
        assert_eq!(catalog().samples.len(), 23);
        assert_eq!(catalog().banks.len(), 3);
        for id in 0..3 {
            assert_eq!(bank(id).instruments.len(), 25);
            assert!(sample(id, 24).is_some());
        }
        assert_eq!(
            sample(0, 9).unwrap().brr.as_ptr(),
            sample(0, 10).unwrap().brr.as_ptr()
        );
    }

    #[test]
    fn bank_echo_seeds_are_addressed_without_spc_ram() {
        assert!(echo_bytes(0, 0xc7ff).is_none());
        assert!(echo_bytes(0, 0xc800).is_some());
        assert_ne!(echo_bytes(0, 0xd000), echo_bytes(1, 0xd000));
        assert_eq!(bank_name(2), "credits");
    }
}
