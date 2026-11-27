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

    /// The parent node, or `ViewId::default()` for the root.
    parent: ViewId,

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

    /// User state created by `state`.
    pub(crate) state: Option<Box<dyn Any>>,

    /// The write generation (`Context::write_gen`) of the last write to `state`, or 0.
    pub(crate) state_gen: u64,

    /// The latest write generation of any state at or under this node. Writing a state raises
    /// this on the node and all its ancestors, so a walk can tell in one check that nothing
    /// under a node changed.
    subtree_gen: u64,

    /// The size a `state` view computed in its last layout, kept while nothing it depends on
    /// changes.
    layout_cache: Option<LayoutCache>,

    /// The accessibility node id a `state` view returned from its last `access`, kept while
    /// nothing it depends on changes.
    access_cache: Option<AccessCache>,

    /// Has an `anim` view been seen at or under this node? Set once and never cleared, which
    /// can only make the `Event::Anim` pass visit more than it needs to.
    has_anim: bool,
}

/// What a `state` view's layout depends on, and the size it computed.
struct LayoutCache {
    /// `Context::write_gen` when the layout started.
    stamp: u64,
    /// States read outside the view's ancestors and subtree.
    foreign: Box<[ViewId]>,
    /// The proposed size.
    proposed: LocalSize,
    size: LocalSize,
}

struct AccessCache {
    /// `Context::write_gen` when the walk started.
    stamp: u64,
    /// States read outside the view's ancestors and subtree.
    foreign: Box<[ViewId]>,
    id: Option<accesskit::NodeId>,
}

impl Node {
    pub(crate) fn new(generation: u32, parent: ViewId, label: u64) -> Self {
        Self {
            generation,
            live: true,
            parent,
            label,
            children: vec![],
            tid: None,
            layout: LayoutBox::default(),
            #[cfg(test)]
            has_layout: false,
            state: None,
            state_gen: 0,
            subtree_gen: 0,
            layout_cache: None,
            access_cache: None,
            has_anim: false,
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

    /// Creates a child of `parent` with the given label.
    fn alloc(&mut self, parent: ViewId, label: u64) -> ViewId {
        if let Some(index) = self.free.pop() {
            let node = &mut self.nodes[index as usize];
            let generation = node.generation;
            *node = Node::new(generation, parent, label);
            ViewId { index, generation }
        } else {
            let index = self.nodes.len() as u32;
            self.nodes.push(Node::new(1, parent, label));
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
        let child = self.alloc(parent, slot as u64);
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
        let child = self.alloc(parent, 0);
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
                let c = by_key
                    .remove(&key)
                    .unwrap_or_else(|| self.alloc(parent, key));
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

    /// Records a write to the state of `id`: raises the state's generation and the subtree
    /// generations of `id` and all its ancestors.
    pub(crate) fn mark_written(&mut self, id: ViewId) {
        self.write_gen += 1;
        let stamp = self.write_gen;
        self.node_mut(id).state_gen = stamp;
        let mut node = id;
        while !node.is_default() {
            let n = &mut self.nodes[node.index as usize];
            n.subtree_gen = stamp;
            node = n.parent;
        }
    }

    /// Has the state of `id` been written since the last update?
    pub(crate) fn is_dirty(&self, id: ViewId) -> bool {
        self.node(id).state_gen > self.clean_gen
    }

    /// Has any state at or under `id` been written since the last update?
    pub(crate) fn is_subtree_dirty(&self, id: ViewId) -> bool {
        self.node(id).subtree_gen > self.clean_gen
    }

    /// Makes every cached layout and accessibility result stale.
    ///
    /// Cached results follow the states a view depends on: those of its enclosing `state`
    /// views, those under it, and any other state it reads through the `Context`. Call this
    /// after changing anything else that views read while building, such as a value in a
    /// `RefCell` or a field of the `Context` like `key_mods`. rui calls it itself for a new
    /// window size or keyboard focus.
    pub fn invalidate_all(&mut self) {
        self.write_gen += 1;
        self.floor_gen = self.write_gen;
        self.set_dirty();
    }

    /// Is a result computed for `id` at generation `stamp` still valid: nothing under `id`, no
    /// enclosing `state` view, and none of the `foreign` states it read written since, and
    /// nothing invalidated everything?
    fn still_valid(&self, node: &Node, stamp: u64, foreign: &[ViewId]) -> bool {
        stamp >= self.floor_gen
            && node.subtree_gen <= stamp
            && self.ancestor_gen <= stamp
            && foreign
                .iter()
                .all(|r| self.is_live(*r) && self.nodes[r.index as usize].state_gen <= stamp)
    }

    /// Starts recording the states read while the `state` view at `id` computes a result to
    /// cache. Returns what `end_reads` needs.
    pub(crate) fn begin_reads(&self) -> usize {
        self.reading.set(self.reading.get() + 1);
        self.reads.borrow().len()
    }

    /// Stops recording for the `state` view at `id`, returning the states read since
    /// `begin_reads` that are neither enclosing it nor under it, whose writes the generations
    /// don't catch. The reads stay recorded for any enclosing `state` view.
    pub(crate) fn end_reads(&self, id: ViewId, start: usize) -> Box<[ViewId]> {
        let depth = self.reading.get() - 1;
        self.reading.set(depth);
        let mut reads = self.reads.borrow_mut();
        let mut foreign: Vec<ViewId> = reads[start..]
            .iter()
            .copied()
            .filter(|r| !self.is_ancestor_or_self(*r, id) && !self.is_ancestor_or_self(id, *r))
            .collect();
        foreign.sort_by_key(|r| (r.index, r.generation));
        foreign.dedup();
        if depth == 0 {
            reads.clear();
        }
        foreign.into_boxed_slice()
    }

    /// Passes the foreign reads of a reused result on to the `state` views computing around it.
    fn reuse_reads(&self, foreign: &[ViewId]) {
        if self.reading.get() > 0 && !foreign.is_empty() {
            self.reads.borrow_mut().extend_from_slice(foreign);
        }
    }

    /// The child of `parent` that is `id` or one of its ancestors, or `None` when `id` isn't
    /// under `parent`.
    pub(crate) fn child_toward(&self, parent: ViewId, id: ViewId) -> Option<ViewId> {
        let mut node = id;
        while self.is_live(node) {
            let up = self.nodes[node.index as usize].parent;
            if up == parent {
                return Some(node);
            }
            node = up;
        }
        None
    }

    /// Is `a` the node `b` or one of its ancestors?
    fn is_ancestor_or_self(&self, a: ViewId, b: ViewId) -> bool {
        let mut node = b;
        while self.is_live(node) {
            if node == a {
                return true;
            }
            node = self.nodes[node.index as usize].parent;
        }
        false
    }

    /// Enters the `state` view at `id` during a walk: the views under it depend on its state.
    /// Returns what `leave_state` needs to restore.
    pub(crate) fn enter_state(&mut self, id: ViewId) -> u64 {
        let before = self.ancestor_gen;
        self.ancestor_gen = before.max(self.node(id).state_gen);
        before
    }

    pub(crate) fn leave_state(&mut self, before: u64) {
        self.ancestor_gen = before;
    }

    /// The size a `state` view at `id` computed for proposed size `proposed`, if it is still
    /// valid, so its layout can be skipped without building its view.
    pub(crate) fn cached_layout(&self, id: ViewId, proposed: LocalSize) -> Option<LocalSize> {
        let node = self.node(id);
        let cache = node.layout_cache.as_ref()?;
        if cache.proposed == proposed && self.still_valid(node, cache.stamp, &cache.foreign) {
            self.reuse_reads(&cache.foreign);
            Some(cache.size)
        } else {
            None
        }
    }

    pub(crate) fn store_layout(
        &mut self,
        id: ViewId,
        stamp: u64,
        foreign: Box<[ViewId]>,
        proposed: LocalSize,
        size: LocalSize,
    ) {
        self.node_mut(id).layout_cache = Some(LayoutCache {
            stamp,
            foreign,
            proposed,
            size,
        });
    }

    /// The accessibility node id a `state` view at `id` returned, if still valid. The nodes
    /// under it were sent in an earlier update and haven't changed.
    pub(crate) fn cached_access(&self, id: ViewId) -> Option<Option<accesskit::NodeId>> {
        let node = self.node(id);
        let cache = node.access_cache.as_ref()?;
        if self.still_valid(node, cache.stamp, &cache.foreign) {
            self.reuse_reads(&cache.foreign);
            Some(cache.id)
        } else {
            None
        }
    }

    pub(crate) fn store_access(
        &mut self,
        id: ViewId,
        stamp: u64,
        foreign: Box<[ViewId]>,
        result: Option<accesskit::NodeId>,
    ) {
        self.node_mut(id).access_cache = Some(AccessCache {
            stamp,
            foreign,
            id: result,
        });
    }

    /// Drops every `state` view's cached access result, so the next `access` walk gives the
    /// whole tree.
    pub(crate) fn clear_access_caches(&mut self) {
        for node in &mut self.nodes {
            node.access_cache = None;
        }
    }

    /// Records that an `anim` view is at `id`.
    pub(crate) fn mark_anim(&mut self, id: ViewId) {
        let mut node = id;
        while !node.is_default() {
            let n = &mut self.nodes[node.index as usize];
            if n.has_anim {
                break;
            }
            n.has_anim = true;
            node = n.parent;
        }
    }

    /// Can the `Event::Anim` pass skip the `state` view at `id`? Only if its last layout visited
    /// everything under it, nothing changed since, and no `anim` view was seen there.
    pub(crate) fn skip_anim(&self, id: ViewId) -> bool {
        let node = self.node(id);
        !node.has_anim
            && node
                .layout_cache
                .as_ref()
                .is_some_and(|c| self.still_valid(node, c.stamp, &c.foreign))
    }

    /// Can the `Event::Anim` pass skip the view at `id`, which need not be a `state` view?
    /// Only if no `anim` view was seen there, and the tree was laid out with nothing written
    /// since, so every `anim` view in it was seen.
    pub(crate) fn skip_unmarked_anim(&self, id: ViewId) -> bool {
        !self.node(id).has_anim && self.marked_gen == Some(self.write_gen)
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
