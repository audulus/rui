use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// `ViewId` is a unique identifier for a view. We're using a u64 and hashing
/// under the assumption there won't be collisions. The underlying u64 is a function
/// of the path down the view tree.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewId {
    pub id: u64,
}

impl ViewId {
    /// Computes the ID for a child using a hashable value. For views
    /// which don't have dynamic children (e.g. `vstack` etc.) the value
    /// will be the integer index of the child. Dynamic
    /// views (e.g. `list`) will hash an item identifier.
    pub fn child<Index: Hash>(&self, index: &Index) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.id);
        index.hash(&mut hasher);
        Self {
            id: hasher.finish(),
        }
    }

    /// Returns the corresponding AccessKit ID. We're assuming
    /// the underlying u64 isn't zero.
    pub fn access_id(&self) -> accesskit::NodeId {
        accesskit::NodeId(std::num::NonZeroU64::new(self.id).unwrap())
    }
}
