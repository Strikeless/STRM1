use super::Memory;

#[test]
fn word_mut_patches_correctly() {
    let magic = 0xABCD;
    let expected_data = libstrmisa::word_to_bytes(magic);

    let mut memory = Memory::new(vec![0; 2]);
    *memory.word_mut(0).unwrap() = magic;

    assert_eq!(
        &memory.0, &expected_data,
        "memory data {:x?} differs from expected {:x?}",
        memory.0, expected_data
    )
}
