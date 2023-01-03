Determine the size in bytes an object occupies inside RAM.

The [`GetSize`] trait can be used to determine the size of an object inside the stack as well as in the heap. The [`size_of`](std::mem::size_of) function provided by the standard library can already be used to determine the size of an object in the stack, but many application (e.g. for caching) do also need to know the number of bytes occupied inside the heap, for which this library provides an appropriate trait.

## Ownership based accounting

This library follows the idea that only bytes owned by a certain object should be accounted for, and not bytes owned by different objects which are only borrowed. This means in particular that objects referenced by pointers are ignored.

### Example

```rust
use get_size::GetSize;
use get_size_derive::*;

#[derive(GetSize)]
struct Test<'a> {
  value: &'a String,
}

fn main() {
  let value = String::from("hello");

  // This string occupies 5 bytes at the heap, but a pointer is treated as not occupying
  // anything at the heap.
  assert_eq!(value.get_heap_size(), 5);
  assert_eq!(GetSize::get_heap_size(&&value), 0); // Fully qualified syntax

  // WARNING: Duo to rust's automatic dereferencing, a simple pointer will be dereferenced
  // to the original value, causing the borrowed bytes to be accounted for too.
  assert_eq!((&value).get_heap_size(), 5);
  // The above gets rewritten by to compiler into:
  // assert_eq!(value.get_heap_size(), 5);

  // Our derive macro uses fully qualified syntax, so auto-dereferencing does
  // not occour.
  let value = Test {
    value: &value,
  };

  // The String is now only borrowed, leading to its heap bytes not being
  // accounted for.
  assert_eq!(value.get_heap_size(), 0);
}
```

On the other hand references implemented as shared ownership are treated as owned values. It is your responsibility to ensure that the bytes occupied by them are not counted twice in your application!

### Example

```rust
use std::sync::Arc;
use get_size::GetSize;

fn main() {
  let value = String::from("hello");
  assert_eq!(value.get_heap_size(), 5);

  // From a technical point of view, Arcs own the data they reference.
  // Given so their heap data gets accounted for too.
  // Note that an Arc does store the String's stack bytes also inside the heap.
  let value = Arc::new(value);
  assert_eq!(value.get_heap_size(), std::mem::size_of::<String>() + 5);
}
```

# How to implement

The [`GetSize`] trait is already implemented for most objects defined by the standard library, like [`Vec`](std::vec::Vec), [`HashMap`](std::collections::HashMap), [`String`] as well as all the primitive values, like [`u8`], [`i32`] etc.

Unless you have a complex datastructure which requires a manual implementation, you can easily derive [`GetSize`] for your own structs and enums. The derived implementation will implement [`GetSize::get_heap_size`] by simply calling [`GetSize::get_heap_size`] on all values contained inside the struct or enum variant and return the sum of them.

You will need to activate the `derive` feature first, which is disabled by default. Add the following to your `cargo.toml`:

```toml
get-size = { version = "^0.1", features = ["derive"] }
```

Then you can easily derive [`GetSize`]. If you want the derive macro to ignore a certain struct field you can add the `ignore` attribute to it. This might be usefull if some values do not implement the [`GetSize`] trait and do not have data on the heap, or if the data on the heap has already been accounted for somewhere else.

```rust
use get_size::GetSize;

#[derive(GetSize)]
pub struct OwnStruct {
    value1: String,
    value2: u64,
    #[get_size(ignore)]
    value3: ExternalStruct,
}

// Does not implement GetSize, so we ignore it.
pub struct ExternalStruct {
    value1: u8,
}

fn main() {
    let test = OwnStruct {
        value1: "Hello".into(),
        value2: 123,
        value3: ExternalStruct{ value1: 255 },
    };

    assert_eq!(test.get_heap_size(), 5);
}
```

As already mentioned you can also derive [`GetSize`] for enums:

```rust
use get_size::GetSize;

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
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test = TestEnum::Variant3;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant4{x: "Hello".into(), y: "world".into()};
    assert_eq!(test.get_heap_size(), 5 + 5);

    let test = TestEnumNumber::One;
    assert_eq!(test.get_heap_size(), 0);
}
```

You can also derive [`GetSize`] on structs and enums with generics. In that case the generated trait implementation will require all generic types to also implement [`GetSize`].

This behavior may be unfavourable if one or more generic types are ignored duo to the corresponding struct field being ignored. In that case you can also use the `ignore` attribute at the struct level to specifiy the generic parameters which shall not be required to implement [`GetSize`] themselves.

# Example

```rust
use get_size::GetSize;
use get_size_derive::*;

#[derive(GetSize)]
#[get_size(ignore(B, C))]
struct TestStructGenericsIgnore<A, B, C> {
    value1: A,
    #[get_size(ignore)]
    value2: B,
    #[get_size(ignore)]
    value3: C,
}

// Does not implement GetSize, so we ignore it.
struct TestStructNoGetSize {
    value: String,
}

fn main() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    // If we did not ignore the C type, the following would not implement
    // GetSize since C does not implement it.
    let test: TestStructGenericsIgnore<String, u64, TestStructNoGetSize> = TestStructGenericsIgnore {
        value1: "Hello".into(),
        value2: 123,
        value3: no_impl,
    };

    assert_eq!(test.get_heap_size(), 5);
}
```

Note that the derive macro does not support unions. You have to manually implement it for them.
