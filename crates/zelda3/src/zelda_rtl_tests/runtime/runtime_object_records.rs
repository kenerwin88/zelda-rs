use crate::game_state::ObjectRecord;

#[test]
fn object_records_keep_the_original_words_and_masks() {
    assert_eq!(ObjectRecord::liftable(0).word(), 0x1010);
    assert_eq!(ObjectRecord::POT.word(), 0x1111);
    assert_eq!(ObjectRecord::liftable(2).word(), 0x1212);
    assert_eq!(ObjectRecord::big_rock_segment(3).word(), 0x2323);
    assert_eq!(ObjectRecord::bombable_floor_segment(1).word(), 0x3131);
    assert_eq!(ObjectRecord::HAMMER_PEG.word(), 0x4040);
    assert_eq!(ObjectRecord::IDLE_PUSH_BLOCK.word(), 0);
    assert_eq!(ObjectRecord::RESTING_ON_PLATE.word(), 5);
    assert_eq!(
        ObjectRecord::VANISHED.advanced(),
        ObjectRecord::IDLE_PUSH_BLOCK
    );
    assert_eq!(ObjectRecord::ARRIVED.advanced(), ObjectRecord::FALLING);
    assert_eq!(
        ObjectRecord::FALLING.settled(),
        ObjectRecord::IDLE_PUSH_BLOCK
    );
    for word in 0..=u16::MAX {
        let record = ObjectRecord::from_word(word);
        assert_eq!(record.word(), word);
        assert_eq!(
            record.liftable_kind(),
            (word & 0xf0f0 == 0x1010).then_some(usize::from(word & 0x0f))
        );
        assert_eq!(
            record.big_rock_segment_index(),
            (word & 0xf0f0 == 0x2020).then_some(usize::from(word & 0x0f))
        );
        assert_eq!(record.is_hammer_peg(), word & 0xf0f0 == 0x4040);
        assert_eq!(record.skips_room_attribute(), word & 0x00f0 == 0x0030);
        assert_eq!(record.kind_index(), usize::from(word & 0x0f));
        assert_eq!(record.is_idle(), word == 0);
        assert_eq!(record.advanced().word(), word.wrapping_add(1));
        assert_eq!(record.settled().word(), word & 0xff00);
    }
}
