use super::*;

#[derive(GetSize)]
pub struct TestStruct {
    value1: String,
    value2: u64,
}

#[test]
fn derive_struct() {
    let test = TestStruct {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
pub enum TestEnum {
    Variant1(u8, u16, u32),
    Variant2(String),
    Variant3(i64, Vec<u16>),
    Variant4(String, i32, Vec<u32>, bool, &'static str),
    Variant5(f64, TestStruct),
    Variant6,
    Variant7{x: String, y: String},
}

#[test]
fn derive_enum() {
    let test = TestEnum::Variant1(1, 2, 3);
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test = TestEnum::Variant3(-12, vec![1, 2, 3]);
    assert_eq!(test.get_heap_size(), 6);

    let test = TestEnum::Variant4("Test".into(), -123, vec![1, 2, 3, 4], false, "Hello world!");
    assert_eq!(test.get_heap_size(), 4 + 16 + 12);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test = TestEnum::Variant5(12.34, test_struct);
    assert_eq!(test.get_heap_size(), 11);

    let test = TestEnum::Variant6;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant7{x: "Hello".into(), y: "world".into()};
    assert_eq!(test.get_heap_size(), 5 + 5);
}

#[derive(GetSize)]
pub enum TestEnum2 {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[test]
fn derive_enum_c_style() {
    let test = TestEnum2::Zero;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum2::One;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum2::Two;
    assert_eq!(test.get_heap_size(), 0);
}
