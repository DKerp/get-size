# get-size-derive

[![Crates.io](https://img.shields.io/crates/v/get-size-derive)](https://crates.io/crates/get-size-derive)
[![docs.rs](https://img.shields.io/docsrs/get-size-derive)](https://docs.rs/get-size-derive)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/DKerp/get-size/blob/main/get-size-derive/LICENSE)

Derives [`GetSize`] for structs and enums.

The derive macro will provide a costum implementation of the [`get_heap_size`] method, which will simply call [`get_heap_size`] on all contained values and add the values up. This implies that all values contained in the struct or enum most implement the [`GetSize`] trait themselves.

When deriving [`GetSize`] for structs you can use the `ignore` attribute to make the derive macro ignore certain values. This might be usefull if some values do not implement the [`GetSize`] trait and do not have data on the heap, or if the data on the heap has already been accounted for somewhere else.

## Example

```rust
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

## Panics

The derive macro will panic if used on unions since these are currently not supported. This might change in the future.

Note that there will be a compilation error if one of the (not ignored) values encountered does not implement the [`GetSize`] trait.

## License

This library is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this library by you, shall be licensed as MIT, without any additional terms or conditions.

[`GetSize`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html
[`get_heap_size`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html#method.get_heap_size
