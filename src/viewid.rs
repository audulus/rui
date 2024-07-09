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
    /// Returns the corresponding AccessKit ID. We're assuming
    /// the underlying u64 isn't zero.
    pub fn access_id(&self) -> accesskit::NodeId {
        accesskit::NodeId(self.id)
    }

    pub fn is_default(self) -> bool {
        self == ViewId::default()
    }
}

pub type IdPath = Vec<u64>;

pub fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    hasher.finish()
}
