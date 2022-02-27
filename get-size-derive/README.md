# get-size-derive

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

## Panics

The derive macro will panic if used on unions since these are currently not supported. This might change in the future.

Note that there will be a compilation error if one of the (not ignored) values encountered does not implement the [`GetSize`] trait.

## License

This library is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this library by you, shall be licensed as MIT, without any additional terms or conditions.

[`GetSize`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html
[`get_heap_size`]: https://docs.rs/get-size/latest/get_size/trait.GetSize.html#method.get_heap_size
