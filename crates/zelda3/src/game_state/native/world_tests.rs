use super::*;

#[test]
fn overworld_map16_source_pages_read_named_wram_pages() {
    let mut ram = vec![0; 0x8000];
    write_le_u16(&mut ram, 0x2000 + 0x010, 0x1234);
    write_le_u16(&mut ram, 0x4000 + 0x010, 0xabcd);

    let decode = OverworldMap16Decode::new(&ram);
    assert_eq!(
        decode.source_page_word(OverworldMap16SourcePage::Main, 0x010),
        0x1234
    );
    assert_eq!(
        decode.source_page_word(OverworldMap16SourcePage::Overlay, 0x010),
        0xabcd
    );
}
