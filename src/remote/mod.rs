//! Contains implementation of `MemoryUsage` for very common external
//! crates. Each of them must be enable with the `enable-<crate-name>`
//! feature.

#[cfg(feature = "indexmap")]
mod indexmap;

#[cfg(feature = "serde_json")]
mod serde_json;
