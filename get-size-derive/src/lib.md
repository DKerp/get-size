Derives [`GetSize`] for structs and enums.

The derive macro will provide a costum implementation of the [`get_heap_size`] method, which will simply call [`get_heap_size`] on all contained values and add the values up. This implies that by default all values contained in the struct or enum must implement the [`GetSize`] trait themselves.

Note that the derive macro _does not support unions_. You have to manually implement it for them.

### Examples

Deriving [`GetSize`] for a struct:

```rust
use get_size::{GetSize, GetSizeNoTracker};

#[derive(GetSize)]
pub struct OwnStruct {
    value1: String,
    value2: u64,
}

fn main() {
    let test = OwnStruct {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5);
}
```

Deriving [`GetSize`] for an enum:

```rust
use get_size::{GetSize, GetSizeNoTracker};

#[derive(GetSize)]
pub enum TestEnum {
    Variant1(u8, u16, u32),
    Variant2(String),
    Variant3,
    Variant4{x: String, y: String},
}

#[derive(GetSize)]
pub enum TestEnumNumber {
    Zero = 0,
    One = 1,
    Two = 2,
}

fn main() {
    let test = TestEnum::Variant1(1, 2, 3);
    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5);

    let test = TestEnum::Variant3;
    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 0);

    let test = TestEnum::Variant4{x: "Hello".into(), y: "world".into()};
    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5 + 5);

    let test = TestEnumNumber::One;
    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 0);
}
```

The derive macro does also work with generics. The generated trait implementation will by default require all generic types to implement [`GetSize`] themselves, but this [can be changed](#ignoring-certain-generic-types).

```rust
use get_size::{GetSize, GetSizeNoTracker};

#[derive(GetSize)]
struct TestStructGenerics<A, B> {
    value1: A,
    value2: B,
}

#[derive(GetSize)]
enum TestEnumGenerics<A, B> {
  Variant1(A),
  Variant2(B),
}

fn main() {
    let test: TestStructGenerics<String, u64> = TestStructGenerics {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5);

    let test = String::from("Hello");
    let test: TestEnumGenerics<String, u64> = TestEnumGenerics::Variant1(test);

    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5);

    let test: TestEnumGenerics<String, u64> = TestEnumGenerics::Variant2(100);

    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 0);
}
```

## Dealing with external types which do not implement GetSize

Deriving [`GetSize`] is straight forward if all the types contained in your data structure implement [`GetSize`] themselves, but this might not always be the case. For that reason the derive macro offers some helpers to assist you in that case.

Note that the helpers are currently only available for regular structs, that is they do neither support tuple structs nor enums.

### Ignoring certain values

You can tell the derive macro to ignore certain struct fields by adding the `ignore` attribute to them. The generated implementation of [`get_heap_size`] will then simple skip this field.

#### Example

The idiomatic use case for this helper is if you use shared ownership and do not want your data to be counted twice.

```rust
use std::sync::Arc;
use get_size::{GetSize, GetSizeNoTracker};

#[derive(GetSize)]
struct PrimaryStore {
  id: u64,
  shared_data: Arc<Vec<u8>>,
}

#[derive(GetSize)]
struct SecondaryStore {
  id: u64,
  #[get_size(ignore)]
  shared_data: Arc<Vec<u8>>,
}

fn main() {
  let shared_data = Arc::new(Vec::with_capacity(1024));

  let primary_data = PrimaryStore {
    id: 1,
    shared_data: Arc::clone(&shared_data),
  };

  let secondary_data = SecondaryStore {
    id: 2,
    shared_data,
  };

  // Note that Arc does also store the Vec's stack data on the heap.
  assert_eq!(primary_data.get_heap_size(&mut GetSizeNoTracker), Vec::<u8>::get_stack_size() + 1024);
  assert_eq!(secondary_data.get_heap_size(&mut GetSizeNoTracker), 0);
}
```

#### Example

But you may also use this as a band aid, if a certain struct fields type does not implement [`GetSize`].

Be aware though that this will result in an implementation which will return incorrect results, unless the heap size of that type is indeed always zero and can thus be ignored. It is therefor advisable to use one of the next two helper options instead.

```rust
use get_size::{GetSize, GetSizeNoTracker};

// Does not implement GetSize!
struct TestStructNoGetSize {
    value: String,
}

// Implements GetSize, even through one field's type does not implement it.
#[derive(GetSize)]
struct TestStruct {
  name: String,
  #[get_size(ignore)]
  ignored_value: TestStructNoGetSize,
}

fn main() {
  let ignored_value = TestStructNoGetSize {
    value: "Hello world!".into(),
  };

  let test = TestStruct {
    name: "Adam".into(),
    ignored_value,
  };

  // Note that the result is lower then it should be.
  assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 4);
}
```

### Returning a fixed value

In same cases you may be dealing with external types which allocate a fixed amount of bytes at the heap. In this case you may use the `size` attribute to always account the given field with a fixed value.

```rust
use get_size::{GetSize, GetSizeNoTracker};
#
# struct Buffer1024 {}
#
# impl Buffer1024 {
#   fn new() -> Self {
#      Self {}
#   }
# }

#[derive(GetSize)]
struct TestStruct {
  id: u64,
  #[get_size(size = 1024)]
  buffer: Buffer1024, // Always allocates exactly 1KB at the heap.
}

fn main() {
  let test = TestStruct {
    id: 1,
    buffer: Buffer1024::new(),
  };

  assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 1024);
}
```

### Using a helper function

In same cases you may be dealing with an external data structure for which you know how to calculate its heap size using its public methods. In that case you may either use the newtype pattern to implement [`GetSize`] for it directly, or you can use the `size_fn` attribute, which will call the given function in order to calculate the fields heap size.

The latter is especially usefull if you can make use of a certain trait to calculate the heap size for multiple types.

Note that unlike in other crates, the name of the function to be called is __not__ encapsulated by double-quotes ("), but rather given directly.

```rust
use get_size::{GetSize, GetSizeNoTracker};
#
# type ExternalVecAlike<T> = Vec<T>;

#[derive(GetSize)]
struct TestStruct {
  id: u64,
  #[get_size(size_fn = vec_alike_helper)]
  buffer: ExternalVecAlike<u8>,
}

// NOTE: We assume that slice.len()==slice.capacity()
fn vec_alike_helper<V, T>(slice: &V) -> usize
where
  V: AsRef<[T]>,
{
  std::mem::size_of::<T>() * slice.as_ref().len()
}

fn main() {
  let buffer = vec![0u8; 512];
  let buffer: ExternalVecAlike<u8> = buffer.into();

  let test = TestStruct {
    id: 1,
    buffer,
  };

  assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 512);
}
```

### Ignoring certain generic types

If your struct uses generics, but the fields at which they are stored are ignored or get handled by helpers because the generic does not implement [`GetSize`], you will have to mark these generics with a special struct level `ignore` attribute. Otherwise the derived [`GetSize`] implementation would still require these generics to implement [`GetSize`], even through there is no need for it.

```rust
use get_size::{GetSize, GetSizeNoTracker};

#[derive(GetSize)]
#[get_size(ignore(B, C, D))]
struct TestStructHelpers<A, B, C, D> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = get_size_helper)]
    value3: C,
    #[get_size(ignore)]
    value4: D,
}

// Does not implement GetSize
struct NoGS {}

fn get_size_helper<C>(_value: &C) -> usize {
    50
}

fn main() {
    let test: TestStructHelpers<String, NoGS, NoGS, u64> = TestStructHelpers {
        value1: "Hello".into(),
        value2: NoGS {},
        value3: NoGS {},
        value4: 123,
    };

    assert_eq!(test.get_heap_size(&mut GetSizeNoTracker), 5 + 100 + 50);
}
```

# Panics

The derive macro will panic if used on unions since these are currently not supported.

Note that there will be a compilation error if one of the (not ignored) values encountered does not implement the [`GetSize`] trait.

[`GetSize`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html
[`get_heap_size`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html#method.get_heap_size
