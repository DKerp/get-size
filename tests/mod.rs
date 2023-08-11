use get_size::*;



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
pub struct TestStructGenerics<A, B> {
    value1: A,
    value2: B,
}

#[test]
fn derive_struct_with_generics() {
    let test: TestStructGenerics<String, u64> = TestStructGenerics {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}


#[derive(GetSize)]
#[get_size(ignore(B, C))]
#[allow(dead_code)]
struct TestStructGenericsIgnore<A, B, C> {
    value1: A,
    #[get_size(ignore)]
    value2: B,
    #[get_size(ignore)]
    value3: C,
}

#[allow(dead_code)]
struct TestStructNoGetSize {
    value: String,
}

#[test]
fn derive_struct_with_generics_and_ignore() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructGenericsIgnore<String, u64, TestStructNoGetSize> = TestStructGenericsIgnore {
        value1: "Hello".into(),
        value2: 123,
        value3: no_impl,
    };

    assert_eq!(test.get_heap_size(), 5);
}


#[derive(GetSize)]
#[get_size(ignore(B, C))]
#[allow(dead_code)]
struct TestStructHelpers<A, B, C> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = get_size_helper)]
    value3: C,
}

fn get_size_helper<C>(_value: &C) -> usize {
    50
}

#[test]
fn derive_struct_with_generics_and_helpers() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructHelpers<String, u64, TestStructNoGetSize> = TestStructHelpers {
        value1: "Hello".into(),
        value2: 123,
        value3: no_impl,
    };

    assert_eq!(test.get_heap_size(), 5 + 100 + 50);
}


#[derive(GetSize)]
pub struct TestStructGenericsLifetimes<'a, A, B> {
    value1: A,
    value2: &'a B,
}

#[test]
fn derive_struct_with_generics_and_lifetimes() {
    let value = 123u64;

    let test: TestStructGenericsLifetimes<String, u64> = TestStructGenericsLifetimes {
        value1: "Hello".into(),
        value2: &value,
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

    let s: String = "Test".into();
    assert_eq!(s.get_heap_size(), 4);
    let v = vec![1, 2, 3, 4];
    assert_eq!(v.get_heap_size(), 16);
    let test = TestEnum::Variant4(s, -123, v, false, "Hello world!");
    assert_eq!(test.get_heap_size(), 4 + 16);

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
pub enum TestEnumGenerics<'a, A, B, C> {
    Variant1(A),
    Variant2(B),
    Variant3(&'a C),
}

#[test]
fn derive_enum_generics() {
    let test: TestEnumGenerics<u64, String, TestStruct> = TestEnumGenerics::Variant1(123);
    assert_eq!(test.get_heap_size(), 0);

    let test: TestEnumGenerics<u64, String, TestStruct> = TestEnumGenerics::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test: TestEnumGenerics<u64, String, TestStruct> = TestEnumGenerics::Variant3(&test_struct);
    assert_eq!(test.get_heap_size(), 0); // It is a pointer.
}


const MINIMAL_NODE_SIZE: usize = 3;

#[derive(Clone, GetSize)]
enum Node<T> where T: Default {
    Block(T),
    Blocks(Box<[T; MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE]>),
    Nodes(Box<[Node<T>; 8]>),
}

#[test]
fn derive_enum_generics_issue1() {
    let test: Node<String> = Node::Block("test".into());
    assert_eq!(test.get_heap_size(), 4);

    let test: Node<u64> = Node::Blocks(Box::new([123; 27]));
    assert_eq!(test.get_heap_size(), 8*27);

    let t1: Node<u64> = Node::Block(123);
    let t2 = t1.clone();
    let t3 = t1.clone();
    let t4 = t1.clone();
    let t5 = t1.clone();
    let t6 = t1.clone();
    let t7 = t1.clone();
    let t8 = t1.clone();
    let test: Node<u64> = Node::Nodes(Box::new([t1,t2,t3,t4,t5,t6,t7,t8]));
    assert_eq!(test.get_heap_size(), 8*std::mem::size_of::<Node<u64>>());
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


#[derive(GetSize)]
pub struct TestNewType(u64);

#[test]
fn derive_newtype() {
    let test = TestNewType(0);
    assert_eq!(u64::get_stack_size(), test.get_size());
}
