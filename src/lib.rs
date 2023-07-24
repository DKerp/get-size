#![doc = include_str!("./lib.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::convert::Infallible;
use std::marker::{PhantomData, PhantomPinned};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::rc::Rc;
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize, Ordering,
};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use get_size_derive::*;

mod remote;

/// Represent a bucket that can track memory addresses that have
/// already been visited by `GetSize`.
pub trait GetSizeTracker {
    /// When first called on a given address returns true, false otherwise.
    fn track(&mut self, address: *const ()) -> bool;
}

impl GetSizeTracker for std::collections::BTreeSet<*const ()> {
    fn track(&mut self, address: *const ()) -> bool {
        self.insert(address)
    }
}

impl GetSizeTracker for std::collections::HashSet<*const ()> {
    fn track(&mut self, address: *const ()) -> bool {
        self.insert(address)
    }
}

pub struct GetSizeNoTracker;

impl GetSizeTracker for GetSizeNoTracker {
    fn track(&mut self, _address: *const ()) -> bool {
        true
    }
}

/// Determine the size in bytes an object occupies inside RAM.
pub trait GetSize: Sized {
    /// Determines how may bytes this object occupies inside the stack.
    ///
    /// The default implementation uses [std::mem::size_of] and should work for almost all types.
    fn get_stack_size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Determines how many bytes this object occupies inside the heap.
    ///
    /// The default implementation returns 0, assuming the object is fully allocated on the stack.
    /// It must be adjusted as appropriate for objects which hold data inside the heap.
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        0
    }

    /// Determines the total size of the object.
    ///
    /// The default implementation simply adds up the result of the other two methods and is not meant
    /// to be changed.
    fn get_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        Self::get_stack_size() + GetSize::get_heap_size(self, tracker)
    }
}

impl GetSize for () {}
impl GetSize for bool {}
impl GetSize for u8 {}
impl GetSize for u16 {}
impl GetSize for u32 {}
impl GetSize for u64 {}
impl GetSize for u128 {}
impl GetSize for usize {}
impl GetSize for NonZeroU8 {}
impl GetSize for NonZeroU16 {}
impl GetSize for NonZeroU32 {}
impl GetSize for NonZeroU64 {}
impl GetSize for NonZeroU128 {}
impl GetSize for NonZeroUsize {}
impl GetSize for i8 {}
impl GetSize for i16 {}
impl GetSize for i32 {}
impl GetSize for i64 {}
impl GetSize for i128 {}
impl GetSize for isize {}
impl GetSize for NonZeroI8 {}
impl GetSize for NonZeroI16 {}
impl GetSize for NonZeroI32 {}
impl GetSize for NonZeroI64 {}
impl GetSize for NonZeroI128 {}
impl GetSize for NonZeroIsize {}
impl GetSize for f32 {}
impl GetSize for f64 {}
impl GetSize for char {}

impl GetSize for AtomicBool {}
impl GetSize for AtomicI8 {}
impl GetSize for AtomicI16 {}
impl GetSize for AtomicI32 {}
impl GetSize for AtomicI64 {}
impl GetSize for AtomicIsize {}
impl GetSize for AtomicU8 {}
impl GetSize for AtomicU16 {}
impl GetSize for AtomicU32 {}
impl GetSize for AtomicU64 {}
impl GetSize for AtomicUsize {}
impl GetSize for Ordering {}

impl GetSize for std::cmp::Ordering {}

impl GetSize for Infallible {}
impl<T> GetSize for PhantomData<T> {}
impl GetSize for PhantomPinned {}

impl GetSize for Instant {}
impl GetSize for Duration {}
impl GetSize for SystemTime {}

impl<'a, T> GetSize for Cow<'a, T>
where
    T: ToOwned,
    <T as ToOwned>::Owned: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        match self {
            Self::Borrowed(_borrowed) => 0,
            Self::Owned(owned) => GetSize::get_heap_size(owned, tracker),
        }
    }
}

#[macro_export]
macro_rules! impl_size_set {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                for v in self.iter() {
                    // We assume that value are hold inside the heap.
                    total += GetSize::get_size(v, tracker);
                }

                let additional: usize = self.capacity() - self.len();
                total += additional * T::get_stack_size();

                total
            }
        }
    };
}

macro_rules! impl_size_set_no_capacity {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                for v in self.iter() {
                    // We assume that value are hold inside the heap.
                    total += GetSize::get_size(v, tracker);
                }

                total
            }
        }
    };
}

#[macro_export]
macro_rules! impl_size_map {
    ($name:ident) => {
        impl<K, V> GetSize for $name<K, V>
        where
            K: GetSize,
            V: GetSize,
        {
            fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                for (k, v) in self.iter() {
                    // We assume that keys and value are hold inside the heap.
                    total += GetSize::get_size(k, tracker);
                    total += GetSize::get_size(v, tracker);
                }

                let additional: usize = self.capacity() - self.len();
                total += additional * K::get_stack_size();
                total += additional * V::get_stack_size();

                total
            }
        }
    };
}

macro_rules! impl_size_map_no_capacity {
    ($name:ident) => {
        impl<K, V> GetSize for $name<K, V>
        where
            K: GetSize,
            V: GetSize,
        {
            fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                for (k, v) in self.iter() {
                    // We assume that keys and value are hold inside the heap.
                    total += GetSize::get_size(k, tracker);
                    total += GetSize::get_size(v, tracker);
                }

                total
            }
        }
    };
}

impl_size_map_no_capacity!(BTreeMap);
impl_size_set_no_capacity!(BTreeSet);
impl_size_set!(BinaryHeap);
impl_size_map!(HashMap);
impl_size_set!(HashSet);
impl_size_set_no_capacity!(LinkedList);
impl_size_set!(VecDeque);

impl_size_set!(Vec);

macro_rules! impl_size_tuple {
    ($($t:ident, $T:ident),+) => {
        impl<$($T,)*> GetSize for ($($T,)*)
        where
            $(
                $T: GetSize,
            )*
        {
            fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                let ($($t,)*) = self;
                $(
                    total += GetSize::get_heap_size($t, tracker);
                )*

                total
            }
        }
    }
}

macro_rules! execute_tuple_macro_16 {
    ($name:ident) => {
        $name!(v1, V1);
        $name!(v1, V1, v2, V2);
        $name!(v1, V1, v2, V2, v3, V3);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10);
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15, v16, V16
        );
    };
}

execute_tuple_macro_16!(impl_size_tuple);

impl<T, const SIZE: usize> GetSize for [T; SIZE]
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let mut total = 0;

        for element in self.iter() {
            // The array stack size already accounts for the stack size of the elements of the array.
            total += GetSize::get_heap_size(element, tracker);
        }

        total
    }
}

impl<T> GetSize for &[T] where T: GetSize {}

impl<T> GetSize for &T {}
impl<T> GetSize for &mut T {}
impl<T> GetSize for *const T {}
impl<T> GetSize for *mut T {}

impl GetSize for Box<str> {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl<T> GetSize for Box<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        GetSize::get_size(&**self, tracker)
    }
}

impl<T> GetSize for Rc<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(Rc::as_ptr(self) as *const ()) {
            GetSize::get_size(&**self, tracker)
        } else {
            0
        }
    }
}

impl<T> GetSize for std::rc::Weak<T>
where
    T: GetSize + ?Sized,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(std::rc::Weak::as_ptr(self) as *const ()) {
            std::rc::Weak::upgrade(self)
                .map(|rc| GetSize::get_size(&*rc, tracker))
                .unwrap_or(0)
        } else {
            0
        }
    }
}

impl GetSize for Arc<str> {
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(Arc::as_ptr(self) as *const ()) {
            self.len()
        } else {
            0
        }
    }
}

impl<T> GetSize for Arc<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(Arc::as_ptr(self) as *const ()) {
            GetSize::get_size(&**self, tracker)
        } else {
            0
        }
    }
}

impl<T> GetSize for std::sync::Weak<T>
where
    T: GetSize + ?Sized,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(std::sync::Weak::as_ptr(self) as *const ()) {
            std::sync::Weak::upgrade(self)
                .map(|arc| GetSize::get_size(&*arc, tracker))
                .unwrap_or(0)
        } else {
            0
        }
    }
}

impl<T> GetSize for Option<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        match self {
            // The options stack size already accounts for the values stack size.
            Some(t) => GetSize::get_heap_size(t, tracker),
            None => 0,
        }
    }
}

impl<T, E> GetSize for Result<T, E>
where
    T: GetSize,
    E: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        match self {
            // The results stack size already accounts for the values stack size.
            Ok(t) => GetSize::get_heap_size(t, tracker),
            Err(e) => GetSize::get_heap_size(e, tracker),
        }
    }
}

impl<T> GetSize for Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a Mutex does hold its data at the stack.
        GetSize::get_heap_size(&*(self.lock().unwrap()), tracker)
    }
}

impl<T> GetSize for RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a RwLock does hold its data at the stack.
        GetSize::get_heap_size(&*(self.read().unwrap()), tracker)
    }
}

impl GetSize for String {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.capacity()
    }
}

impl GetSize for &str {}

impl GetSize for std::ffi::CString {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.as_bytes_with_nul().len()
    }
}

impl GetSize for &std::ffi::CStr {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.to_bytes_with_nul().len()
    }
}

impl GetSize for std::ffi::OsString {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl GetSize for &std::ffi::OsStr {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl GetSize for std::fs::DirBuilder {}
impl GetSize for std::fs::DirEntry {}
impl GetSize for std::fs::File {}
impl GetSize for std::fs::FileType {}
impl GetSize for std::fs::Metadata {}
impl GetSize for std::fs::OpenOptions {}
impl GetSize for std::fs::Permissions {}
impl GetSize for std::fs::ReadDir {}

impl<T> GetSize for std::io::BufReader<T>
where
    T: GetSize,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let mut total = GetSize::get_heap_size(self.get_ref(), tracker);

        total += self.capacity();

        total
    }
}

impl<T> GetSize for std::io::BufWriter<T>
where
    T: GetSize + std::io::Write,
{
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let mut total = GetSize::get_heap_size(self.get_ref(), tracker);

        total += self.capacity();

        total
    }
}

impl GetSize for std::path::PathBuf {
    fn get_heap_size(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.capacity()
    }
}

impl GetSize for &std::path::Path {}

impl<T> GetSize for Box<[T]> {
    fn get_heap_size(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let mut total = 0;
        for item in self.iter() {
            total += item.get_size(tracker)
        }

        total
    }
}
