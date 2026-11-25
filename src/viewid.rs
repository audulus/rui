use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// `ViewId` is a unique identifier for a view. The underlying u64 is a function of the path down
/// the view tree: each step mixes the parent's id with the child's index (see `IdPath::push`), so
/// distinct paths get ids that behave like independent random 64-bit numbers. We assume they
/// don't collide: among n live views, the chance of any collision is about n² / 2⁶⁵.
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

/// The path from the root of the view tree to a view: the index of each child taken on the way
/// down, with the id of every prefix kept alongside, so the id of the current view is known
/// without hashing the whole path.
#[derive(Clone, Default)]
pub struct IdPath {
    indices: Vec<u64>,
    ids: Vec<u64>,
}

impl IdPath {
    /// The path of the root view.
    pub fn root() -> Self {
        let mut path = Self::default();
        path.push(0);
        path
    }

    /// Steps down to child `index`.
    pub fn push(&mut self, index: u64) {
        let parent = self.ids.last().copied().unwrap_or(0);
        self.indices.push(index);
        self.ids.push(mix(parent, index));
    }

    /// Steps back up to the parent.
    pub fn pop(&mut self) {
        self.indices.pop();
        self.ids.pop();
    }

    /// The depth of the path.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// The id of the view at the end of the path.
    pub fn id(&self) -> ViewId {
        ViewId {
            id: self.ids.last().copied().unwrap_or(0),
        }
    }

    /// The child indices from the root.
    pub fn indices(&self) -> &[u64] {
        &self.indices
    }
}

impl From<Vec<u64>> for IdPath {
    fn from(indices: Vec<u64>) -> Self {
        let mut path = Self::default();
        for index in indices {
            path.push(index);
        }
        path
    }
}

impl PartialEq for IdPath {
    fn eq(&self, other: &Self) -> bool {
        self.indices == other.indices
    }
}

impl Eq for IdPath {}

impl PartialOrd for IdPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IdPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.indices.cmp(&other.indices)
    }
}

impl std::fmt::Debug for IdPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices.fmt(f)
    }
}

/// The finalizer of SplitMix64: a bijection on u64 in which every input bit affects every
/// output bit.
fn fmix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Returns the id of child `index` of the view with id `parent`. For one parent, distinct
/// indices give distinct ids; across parents, ids are as good as random. Never zero, which
/// `ViewId::default` reserves.
fn mix(parent: u64, index: u64) -> u64 {
    match fmix(parent ^ fmix(index.wrapping_add(0x9e37_79b9_7f4a_7c15))) {
        0 => 1,
        id => id,
    }
}

/// A hasher for keys that are already well-mixed u64 ids: it passes the id through.
#[derive(Default)]
pub struct IdHasher(u64);

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }

    fn write(&mut self, bytes: &[u8]) {
        // Not used by `ViewId`, but stay correct for other keys.
        for &byte in bytes {
            self.0 = fmix(self.0 ^ u64::from(byte));
        }
    }
}

/// A map keyed by view id.
pub type IdMap<V> = HashMap<ViewId, V, BuildHasherDefault<IdHasher>>;
/// A set of view ids.
pub type IdSet = HashSet<ViewId, BuildHasherDefault<IdHasher>>;

pub fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_follow_the_path() {
        let mut a = IdPath::root();
        a.push(3);
        a.push(1);
        let b = IdPath::from(vec![0, 3, 1]);
        assert_eq!(a.id(), b.id());
        a.pop();
        assert_eq!(a.id(), IdPath::from(vec![0, 3]).id());
    }

    #[test]
    fn test_structurally_similar_paths_differ() {
        let paths: Vec<Vec<u64>> = vec![
            vec![0],
            vec![0, 0],
            vec![0, 1],
            vec![0, 0, 1],
            vec![0, 1, 0],
            vec![1],
            vec![3],
            vec![1, 2],
            vec![2, 1],
        ];
        let ids: HashSet<u64> = paths
            .iter()
            .map(|p| IdPath::from(p.clone()).id().id)
            .collect();
        assert_eq!(ids.len(), paths.len());
    }

    #[test]
    fn test_no_collisions_in_a_large_tree() {
        // Every path of depth 4 under the root with 40 children per level: 2.6 million ids.
        let mut ids = HashSet::new();
        let mut path = IdPath::root();
        for a in 0..40 {
            path.push(a);
            for b in 0..40 {
                path.push(b);
                for c in 0..40 {
                    path.push(c);
                    for d in 0..40 {
                        path.push(d);
                        assert!(ids.insert(path.id().id));
                        path.pop();
                    }
                    path.pop();
                }
                path.pop();
            }
            path.pop();
        }
    }
}
