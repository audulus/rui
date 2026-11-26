//! `Node`: what the `Context` stores for one view.

use crate::*;
use std::any::{Any, TypeId};

/// Everything the `Context` stores for one view: one node of the tree that mirrors the views.
/// Nodes live in an arena (`Context::nodes`) and are addressed by `ViewId`.
pub(crate) struct Node {
    /// The generation of the slot. Never zero; it changes when the node is freed, so the ids
    /// of freed nodes go stale.
    pub(crate) generation: u32,

    /// Is the slot in use?
    pub(crate) live: bool,

    /// The parent node, or `ViewId::default()` for the root.
    pub(crate) parent: ViewId,

    /// Identifies the node among its parent's children: the child index for stacks and
    /// modifiers, the hashed key for list items. Used to rebuild paths for tests.
    pub(crate) label: u64,

    /// Child nodes, indexed by slot.
    pub(crate) children: Vec<ViewId>,

    /// Type of the view owning the node, where it can change (children of `any_view`).
    pub(crate) tid: Option<TypeId>,

    /// Layout information.
    pub(crate) layout: LayoutBox,

    /// Has layout been set? For snapshot tests.
    #[cfg(test)]
    pub(crate) has_layout: bool,

    /// User state created by `state`.
    pub(crate) state: Option<Box<dyn Any>>,

    /// The write generation (`Context::write_gen`) of the last write to `state`, or 0.
    pub(crate) state_gen: u64,

    /// The latest write generation of any state at or under this node. Writing a state raises
    /// this on the node and all its ancestors, so a walk can tell in one check that nothing
    /// under a node changed.
    pub(crate) subtree_gen: u64,

    /// The size a `state` view computed in its last layout, kept while nothing it depends on
    /// changes.
    pub(crate) layout_cache: Option<LayoutCache>,

    /// The accessibility node id a `state` view returned from its last `access`, kept while
    /// nothing it depends on changes.
    pub(crate) access_cache: Option<AccessCache>,

    /// Has an `anim` view been seen at or under this node? Set once and never cleared, which
    /// can only make the `Event::Anim` pass visit more than it needs to.
    pub(crate) has_anim: bool,
}

/// What a `state` view's layout depends on, and the size it computed.
pub(crate) struct LayoutCache {
    /// `Context::write_gen` when the layout started.
    pub(crate) stamp: u64,
    /// States read outside the view's ancestors and subtree.
    pub(crate) foreign: Box<[ViewId]>,
    /// The proposed size.
    pub(crate) proposed: LocalSize,
    pub(crate) size: LocalSize,
}

pub(crate) struct AccessCache {
    /// `Context::write_gen` when the walk started.
    pub(crate) stamp: u64,
    /// States read outside the view's ancestors and subtree.
    pub(crate) foreign: Box<[ViewId]>,
    pub(crate) id: Option<accesskit::NodeId>,
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
