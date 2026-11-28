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

/// Hashes a value, as for the keys of a `list`. A list may hash all its keys, so this is a fast
/// multiply and xor-shift per word, not the default SipHash: keys come from the app, not from
/// an adversary. Each step is a bijection of the state, so distinct integer keys never share a
/// node, and the xor-shift keeps keys of several words (strings, tuples) from combining
/// linearly, which would make them collide far more often than random.
pub(crate) fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = KeyHasher(0);
    index.hash(&mut hasher);
    hasher.finish()
}

struct KeyHasher(u64);

impl KeyHasher {
    const K: u64 = 0xf135_7aea_2e62_a9c5;

    fn add(&mut self, word: u64) {
        let x = (self.0 ^ word).wrapping_mul(Self::K);
        self.0 = x ^ (x >> 32);
    }
}

impl Hasher for KeyHasher {
    fn write(&mut self, bytes: &[u8]) {
        let (chunks, rest) = bytes.as_chunks::<8>();
        for chunk in chunks {
            self.add(u64::from_le_bytes(*chunk));
        }
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            // Tell the remainder's length apart from trailing zero bytes.
            self.add(u64::from_le_bytes(word) ^ ((rest.len() as u64) << 59));
        }
    }

    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }

    fn write_u16(&mut self, i: u16) {
        self.add(i as u64);
    }

    fn write_u32(&mut self, i: u32) {
        self.add(i as u64);
    }

    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    fn finish(&self) -> u64 {
        self.0.rotate_left(26)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_hashes_differ() {
        let ints: std::collections::HashSet<u64> = (0..10_000usize).map(|i| hh(&i)).collect();
        assert_eq!(ints.len(), 10_000);
        let strs = [
            "",
            "a",
            "a\0",
            "ab",
            "ba",
            "abcdefgh",
            "abcdefgh\0",
            "abcdefghi",
        ];
        let hashes: std::collections::HashSet<u64> = strs.iter().map(hh).collect();
        assert_eq!(hashes.len(), strs.len());
        assert_ne!(hh(&(1, 2)), hh(&(2, 1)));
        // Two-word keys whose words differ in a way a linear hash maps to equal states.
        let a = (1u64, 0u64);
        let b = (0u64, KeyHasher::K);
        assert_ne!(hh(&a), hh(&b));
    }

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
