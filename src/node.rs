//! The tree of nodes that stores per-view data, and the `Context` methods that walk and edit it.

use crate::*;
use std::any::{Any, TypeId};

/// Everything the `Context` stores for one view: one node of the tree that mirrors the views.
/// Nodes live in an arena (`Context::nodes`) and are addressed by `ViewId`.
pub(crate) struct Node {
    /// The generation of the slot. Never zero; it changes when the node is freed, so the ids
    /// of freed nodes go stale.
    generation: u32,

    /// Is the slot in use?
    live: bool,

    /// Identifies the node among its parent's children: the child index for stacks and
    /// modifiers, the hashed key for list items. Used to rebuild paths for tests.
    label: u64,

    /// Child nodes, indexed by slot.
    children: Vec<ViewId>,

    /// Type of the view owning the node, where it can change (children of `any_view`).
    tid: Option<TypeId>,

    /// Layout information.
    pub(crate) layout: LayoutBox,

    /// Has layout been set? For snapshot tests.
    #[cfg(test)]
    has_layout: bool,

    /// The `Context::layout_epoch` in which a `state` view last computed its layout, so a
    /// window resize can invalidate cached layouts.
    pub(crate) layout_epoch: u64,

    /// User state created by `state`.
    pub(crate) state: Option<Box<dyn Any>>,

    /// Has the state been written since the last update?
    pub(crate) state_dirty: bool,
}

impl Node {
    pub(crate) fn new(generation: u32, label: u64) -> Self {
        Self {
            generation,
            live: true,
            label,
            children: vec![],
            tid: None,
            layout: LayoutBox::default(),
            #[cfg(test)]
            has_layout: false,
            layout_epoch: 0,
            state: None,
            state_dirty: false,
        }
    }
}

impl Context {
    /// The id of the root view: what to pass to the root view's `DynView` methods when driving
    /// them directly, as for rendering offscreen.
    pub fn root_id(&self) -> ViewId {
        ViewId {
            index: 0,
            generation: self.nodes[0].generation,
        }
    }

    /// Does `id` refer to a node that hasn't been freed?
    pub(crate) fn is_live(&self, id: ViewId) -> bool {
        self.nodes
            .get(id.index as usize)
            .is_some_and(|node| node.live && node.generation == id.generation)
    }

    pub(crate) fn node(&self, id: ViewId) -> &Node {
        let node = &self.nodes[id.index as usize];
        assert!(
            node.live && node.generation == id.generation,
            "rui: a view id or state handle refers to a view that no longer exists"
        );
        node
    }

    pub(crate) fn node_mut(&mut self, id: ViewId) -> &mut Node {
        let node = &mut self.nodes[id.index as usize];
        assert!(
            node.live && node.generation == id.generation,
            "rui: a view id or state handle refers to a view that no longer exists"
        );
        node
    }

    /// Creates a node with the given label.
    fn alloc(&mut self, label: u64) -> ViewId {
        if let Some(index) = self.free.pop() {
            let node = &mut self.nodes[index as usize];
            let generation = node.generation;
            *node = Node::new(generation, label);
            ViewId { index, generation }
        } else {
            let index = self.nodes.len() as u32;
            self.nodes.push(Node::new(1, label));
            ViewId {
                index,
                generation: 1,
            }
        }
    }

    /// Frees a node and everything under it. Ids of freed nodes go stale.
    fn free_subtree(&mut self, id: ViewId) {
        if !self.is_live(id) {
            // Already freed, as for a key that appears twice in a list.
            return;
        }

        // Release what the view held: nothing would match its id again, so a touch would
        // block other drags with that pointer, and the focus would go nowhere.
        for touch in &mut self.touches {
            if *touch == id {
                *touch = ViewId::default();
                // Only a drag holds the grab, and only while it holds a touch.
                self.grab_cursor = false;
            }
        }
        if self.focused_id == Some(id) {
            self.focused_id = None;
        }

        let node = &mut self.nodes[id.index as usize];
        let children = std::mem::take(&mut node.children);
        node.live = false;
        node.state = None;
        node.generation = node.generation.wrapping_add(1).max(1);
        self.free.push(id.index);
        for child in children {
            self.free_subtree(child);
        }
    }

    /// Returns child `slot` of `parent`, creating it the first time.
    pub(crate) fn child(&mut self, parent: ViewId, slot: usize) -> ViewId {
        if let Some(&child) = self.node(parent).children.get(slot)
            && !child.is_default()
        {
            return child;
        }
        let child = self.alloc(slot as u64);
        let children = &mut self.node_mut(parent).children;
        if children.len() <= slot {
            children.resize(slot + 1, ViewId::default());
        }
        children[slot] = child;
        child
    }

    /// Returns child `slot` of `parent`, freeing any other children, as for the branch
    /// `cond` shows.
    pub(crate) fn sole_child(&mut self, parent: ViewId, slot: usize) -> ViewId {
        let others: Vec<ViewId> = self
            .node(parent)
            .children
            .iter()
            .enumerate()
            .filter(|(i, c)| *i != slot && !c.is_default())
            .map(|(_, c)| *c)
            .collect();
        if !others.is_empty() {
            for other in others {
                self.free_subtree(other);
            }
            for (i, c) in self.node_mut(parent).children.iter_mut().enumerate() {
                if i != slot {
                    *c = ViewId::default();
                }
            }
        }
        self.child(parent, slot)
    }

    /// Returns the only child of `parent`, for a view of type `tid`. If the child was made for a
    /// view of another type, it is freed and made again, so state doesn't carry over to an
    /// unrelated view.
    pub(crate) fn typed_child(&mut self, parent: ViewId, tid: TypeId) -> ViewId {
        if let Some(&child) = self.node(parent).children.first()
            && !child.is_default()
        {
            if self.node(child).tid == Some(tid) {
                return child;
            }
            self.free_subtree(child);
        }
        let child = self.alloc(hh(&tid));
        self.node_mut(child).tid = Some(tid);
        let children = &mut self.node_mut(parent).children;
        children.clear();
        children.push(child);
        child
    }

    /// Matches the children of `parent` to a list's keys, so that afterwards child `i` (see
    /// `child`) is the node for `keys[i]`. Nodes of keys that persist are kept, wherever they
    /// moved; nodes of keys that are gone are freed. Equal keys share a node.
    pub(crate) fn sync_keyed_children<I>(&mut self, parent: ViewId, keys: I)
    where
        I: ExactSizeIterator<Item = u64> + Clone,
    {
        // Fast path: the keys are unchanged.
        {
            let node = self.node(parent);
            if node.children.len() == keys.len()
                && node
                    .children
                    .iter()
                    .zip(keys.clone())
                    .all(|(c, k)| self.nodes[c.index as usize].label == k)
            {
                return;
            }
        }

        let old = std::mem::take(&mut self.node_mut(parent).children);
        let mut by_key: std::collections::HashMap<u64, ViewId> = old
            .iter()
            .map(|c| (self.nodes[c.index as usize].label, *c))
            .collect();
        let mut used = std::collections::HashMap::<u64, ViewId>::new();
        let mut children = Vec::with_capacity(keys.len());
        for key in keys {
            let child = if let Some(&c) = used.get(&key) {
                c
            } else {
                let c = by_key.remove(&key).unwrap_or_else(|| self.alloc(key));
                used.insert(key, c);
                c
            };
            children.push(child);
        }
        for (_, gone) in by_key {
            self.free_subtree(gone);
        }
        self.node_mut(parent).children = children;
    }

    /// Is the state of `id`, or of any view under it, dirty?
    pub(crate) fn subtree_dirty(&self, id: ViewId) -> bool {
        let node = &self.nodes[id.index as usize];
        node.state_dirty
            || node
                .children
                .iter()
                .any(|c| !c.is_default() && self.subtree_dirty(*c))
    }

    pub(crate) fn get_layout(&self, id: ViewId) -> LayoutBox {
        self.node(id).layout
    }

    pub(crate) fn update_layout(&mut self, id: ViewId, layout_box: LayoutBox) {
        let node = self.node_mut(id);
        node.layout = layout_box;
        #[cfg(test)]
        {
            node.has_layout = true;
        }
    }

    pub(crate) fn set_layout_offset(&mut self, id: ViewId, offset: LocalOffset) {
        let node = self.node_mut(id);
        node.layout.offset = offset;
        #[cfg(test)]
        {
            node.has_layout = true;
        }
    }

    pub(crate) fn layout_epoch_of(&self, id: ViewId) -> u64 {
        self.node(id).layout_epoch
    }

    pub(crate) fn set_layout_epoch(&mut self, id: ViewId) {
        let epoch = self.layout_epoch;
        self.node_mut(id).layout_epoch = epoch;
    }

    /// Returns all layout entries with their paths (the labels from the root), sorted by path,
    /// for use in snapshot tests.
    #[cfg(test)]
    pub(crate) fn layout_entries(&self) -> Vec<(Vec<u64>, LayoutBox)> {
        fn walk(
            cx: &Context,
            id: ViewId,
            path: &mut Vec<u64>,
            out: &mut Vec<(Vec<u64>, LayoutBox)>,
        ) {
            let node = cx.node(id);
            path.push(node.label);
            if node.has_layout {
                out.push((path.clone(), node.layout));
            }
            for child in &node.children {
                if !child.is_default() {
                    walk(cx, *child, path, out);
                }
            }
            path.pop();
        }
        let mut entries = vec![];
        walk(self, self.root_id(), &mut vec![], &mut entries);
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries.dedup_by(|a, b| a.0 == b.0);
        entries
    }

    /// Returns the id of the view at `path`, a list of labels from the root (whose label is 0),
    /// for tests.
    #[cfg(test)]
    pub(crate) fn id_at(&self, path: &[u64]) -> ViewId {
        assert_eq!(path.first(), Some(&0));
        let mut id = self.root_id();
        for label in &path[1..] {
            id = *self
                .node(id)
                .children
                .iter()
                .find(|c| !c.is_default() && self.nodes[c.index as usize].label == *label)
                .expect("no view at path");
        }
        id
    }

    /// The number of live nodes, for tests.
    #[cfg(test)]
    pub(crate) fn live_nodes(&self) -> usize {
        self.nodes.len() - self.free.len()
    }
}
