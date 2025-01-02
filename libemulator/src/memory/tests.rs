use crate::memory::Memory;

#[test]
fn word_returns_expected() {
    let expected_word = 0xABCD;
    let word_bytes = libisa::word_to_bytes(expected_word);

    let volatile =
        Memory::<{ libisa::BYTES_PER_WORD }>::new_with_data(word_bytes).expect("Creating volatile");

    let actual_word = volatile.word(0).expect("Getting word");

    assert_eq!(
        *actual_word, expected_word,
        "Got word {:x?} differs from expected {:x?}",
        *actual_word, word_bytes
    );
}

#[test]
fn word_mut_patches_correctly() {
    let magic_word = 0xABCD;
    let expected_bytes = libisa::word_to_bytes(magic_word);

    let mut volatile = Memory::<{ libisa::BYTES_PER_WORD }>::new();
    *volatile.word_mut(0).unwrap() = magic_word;

    let actual_bytes: Vec<_> = volatile.iter_bytes().map(|data_ref| *data_ref).collect();

    assert_eq!(
        &actual_bytes, &expected_bytes,
        "memory data {:x?} differs from expected {:x?}",
        actual_bytes, expected_bytes
    );
}
