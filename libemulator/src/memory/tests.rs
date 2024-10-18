use crate::tracing::none::NoTraceData;

use super::Memory;

#[test]
fn word_mut_patches_correctly() {
    let magic = 0xABCD;
    let expected_data = libstrmisa::word_to_bytes(magic);

    let mut memory = Memory::<NoTraceData>::new(vec![0; 2]);
    *memory.word_mut_untraced(0).unwrap() = magic;

    let memory_data: Vec<_> = memory.iter_untraced().map(|data_ref| *data_ref).collect();

    assert_eq!(
        &memory_data, &expected_data,
        "memory data {:x?} differs from expected {:x?}",
        memory_data, expected_data
    )
}
