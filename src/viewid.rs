use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// `ViewId` identifies a view's node in the `Context`'s node tree: an index into the node
/// arena plus the generation of that slot. When a node is freed its slot's generation changes,
/// so an id that outlives its node (in a `StateHandle` sent through `on_main`, say) is detected
/// as stale instead of reading whatever reuses the slot.
///
/// A node is created the first time a view at that position in the tree is visited, and freed
/// when the view disappears (a list item whose key is gone, the other branch of a `cond`, an
/// `any_view` whose type changed), so state persists exactly as long as its view.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewId {
    pub(crate) index: u32,
    /// Never zero for a live node, so `ViewId::default()` refers to no node.
    pub(crate) generation: u32,
}

impl ViewId {
    /// Returns the corresponding AccessKit ID: the index and generation packed into a u64,
    /// never zero.
    pub fn access_id(&self) -> accesskit::NodeId {
        accesskit::NodeId(((self.generation as u64) << 32) | (self.index as u64 + 1))
    }

    /// Is this the id of no view?
    pub fn is_default(self) -> bool {
        self == ViewId::default()
    }
}

/// Hashes a value, as for the keys of a `list`.
pub fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_ids_differ_and_are_nonzero() {
        let a = ViewId {
            index: 0,
            generation: 1,
        };
        let b = ViewId {
            index: 0,
            generation: 2,
        };
        let c = ViewId {
            index: 1,
            generation: 1,
        };
        assert_ne!(a.access_id(), b.access_id());
        assert_ne!(a.access_id(), c.access_id());
        assert_ne!(ViewId::default().access_id().0, 0);
    }
}
