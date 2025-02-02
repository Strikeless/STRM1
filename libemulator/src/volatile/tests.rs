use number_bytes::NumberBytes;

use crate::volatile::Volatile;

#[test]
fn get_multi() {
    type Multi = u32;

    let multi_expected: Multi = 0xA0B1C2D3;

    let bytes = multi_expected.to_be_bytes();
    let volatile = Volatile::new_with_data(bytes, bytes.len()).unwrap();

    let multi_actual = *volatile.get_multi::<Multi>(0).unwrap();

    assert_eq!(
        multi_expected, multi_actual,
        "Expected {:X?}, got {:X?}",
        multi_expected, multi_actual,
    );
}

#[test]
fn mut_multi_patches_correctly() {
    type Multi = u32;
    let mut volatile = Volatile::<u8, _>::new(Multi::BYTES);

    let multi_value: Multi = 0xA0B1C2D3;
    *volatile.get_mut_multi(0).unwrap() = multi_value;

    let data_expected = multi_value.to_be_bytes();
    let data_actual = volatile.data.as_slice();

    assert_eq!(
        data_expected, data_actual,
        "Expected {:X?}, got {:X?}",
        data_expected, data_actual,
    )
}
