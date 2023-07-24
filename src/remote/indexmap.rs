use crate::{impl_size_map, impl_size_set, GetSize, GetSizeTracker};
use indexmap::{IndexMap, IndexSet};

impl_size_map!(IndexMap);
impl_size_set!(IndexSet);
