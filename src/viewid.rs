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
        accesskit::NodeId(std::num::NonZeroU128::new(self.id as u128).unwrap())
    }

    pub fn is_default(self) -> bool {
        self == ViewId::default()
    }
}

pub type IdPath = Vec<u64>;

// Temporary function so we don't have to change too much at once.
// Just adding Id paths for now to the View functions.
pub fn hash(path: &IdPath) -> ViewId {
    let mut hasher = DefaultHasher::new();
    for id in path {
        hasher.write_u64(*id);
    }
    ViewId {
        id: hasher.finish(),
    }
}

pub fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    hasher.finish()
}
