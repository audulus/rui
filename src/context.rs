use crate::*;
use euclid::*;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::ops;
use std::sync::Arc;

pub type LocalSpace = vger::defs::LocalSpace;
pub type WorldSpace = vger::defs::WorldSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;
pub type WorldPoint = Point2D<f32, WorldSpace>;
pub type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;
pub type WorldToLocal = Transform2D<f32, WorldSpace, LocalSpace>;

#[derive(Clone, Eq, PartialEq)]
pub struct CommandInfo {
    pub path: Arc<str>,
    pub key: Option<HotKey>,
}

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

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
    fn new(generation: u32, label: u64) -> Self {
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

pub(crate) type EnvMap = HashMap<TypeId, Box<dyn Any>>;

pub struct RenderInfo<'a, 'window> {
    pub device: &'a wgpu::Device,
    pub surface: &'a wgpu::Surface<'window>,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub queue: &'a wgpu::Queue,
}

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Per-view storage: a tree of nodes in an arena. Slot 0 is the root.
    nodes: Vec<Node>,

    /// Indices of freed slots in `nodes`.
    free: Vec<u32>,

    /// Nodes whose state was written since the last update.
    dirty_nodes: Vec<ViewId>,

    /// Bumped to invalidate every cached layout, as when the window resizes.
    pub(crate) layout_epoch: u64,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewId; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [LocalPoint; 16],

    /// Current mouse button for event handling.
    pub(crate) mouse_button: Option<MouseButton>,

    /// Mouse button state.
    pub mouse_buttons: MouseButtons,

    /// Keyboard modifiers state.
    pub key_mods: KeyboardModifiers,

    /// The view that has the keyboard focus.
    pub(crate) focused_id: Option<ViewId>,

    /// The current title of the window
    pub window_title: Arc<str>,

    /// Are we fullscreen?
    pub fullscreen: bool,

    /// Has the state changed?
    pub(crate) dirty: bool,

    /// Are we currently setting the dirty bit?
    pub(crate) enable_dirty: bool,

    /// Values indexed by type.
    pub(crate) env: EnvMap,

    /// Regions of window that needs repainting.
    pub(crate) dirty_region: Region<WorldSpace>,

    /// The `state` views whose layout is being computed, innermost last. A `state` view's
    /// layout depends on the states of these ancestors.
    pub(crate) id_stack: Vec<ViewId>,

    /// Previous window size.
    window_size: Size2D<f32, WorldSpace>,

    /// Offset for events at the root level.
    root_offset: LocalOffset,

    /// While drawing, the part of the window that can show anything, in world space: the window
    /// narrowed by the clips and scroll views around the view being drawn. Views outside it may
    /// skip drawing. None draws everything.
    pub(crate) visible: Option<WorldRect>,

    /// Render the dirty rectangle for debugging?
    render_dirty: bool,

    /// Lock the cursor in position. Useful for dragging knobs.
    pub(crate) grab_cursor: bool,

    /// Value of grab_cursor before processing event.
    pub(crate) prev_grab_cursor: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new(1, 0)],
            free: vec![],
            dirty_nodes: vec![],
            layout_epoch: 1,
            touches: [ViewId::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            mouse_button: None,
            mouse_buttons: Default::default(),
            key_mods: Default::default(),
            focused_id: None,
            window_title: "rui".into(),
            fullscreen: false,
            dirty: false,
            enable_dirty: true,
            env: HashMap::new(),
            dirty_region: Region::EMPTY,
            id_stack: vec![],
            window_size: Size2D::default(),
            root_offset: LocalOffset::zero(),
            visible: None,
            render_dirty: false,
            grab_cursor: false,
            prev_grab_cursor: false,
        }
    }

    /// Call this after the event queue is cleared.
    pub fn update(
        &mut self,
        view: &impl View,
        vger: &mut Vger,
        access_nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        window_size: Size2D<f32, WorldSpace>,
    ) -> bool {
        // If the window size has changed, force a relayout.
        if window_size != self.window_size {
            self.layout_epoch += 1;
            self.window_size = window_size;
        }

        let root = self.root_id();

        // Run any animations.
        let mut actions = vec![];
        view.process(&Event::Anim, root, self, &mut actions);

        if self.dirty {
            // Get a new accesskit tree.
            let mut nodes = vec![];

            view.access(root, self, &mut nodes);

            if nodes != *access_nodes {
                log::debug!("access nodes:");
                for (id, node) in &nodes {
                    log::debug!(
                        "  id: {:?} role: {:?}, children: {:?}",
                        id,
                        node.role(),
                        node.children()
                    );
                }
                *access_nodes = nodes;
            } else {
                // log::debug!("access nodes unchanged");
            }

            // XXX: we're doing layout both here and in rendering.
            view.layout(
                root,
                &mut LayoutArgs {
                    sz: [window_size.width, window_size.height].into(),
                    cx: self,
                    text_bounds: &mut |str, size, max_width| vger.text_bounds(str, size, max_width),
                },
            );

            // Get dirty rectangles.
            view.dirty(root, LocalToWorld::identity(), self);

            self.clear_dirty();

            true
        } else {
            false
        }
    }

    /// Redraw the UI using wgpu.
    pub fn render(
        &mut self,
        render_info: RenderInfo,
        view: &impl View,
        vger: &mut Vger,
        window_size: Size2D<f32, WorldSpace>,
        scale: f32,
    ) {
        let surface = render_info.surface;
        let device = render_info.device;
        let config = render_info.config;
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                surface.configure(device, config);
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };

        vger.begin(window_size.width, window_size.height, scale);

        let root = self.root_id();
        // Disable dirtying the state during layout and rendering
        // to avoid constantly re-rendering if some state is saved.
        self.enable_dirty = false;
        let local_window_size = window_size.cast_unit::<LocalSpace>();
        let sz = view.layout(
            root,
            &mut LayoutArgs {
                sz: local_window_size,
                cx: self,
                text_bounds: &mut |str, size, max_width| vger.text_bounds(str, size, max_width),
            },
        );

        // Center the root view in the window.
        self.root_offset = ((local_window_size - sz) / 2.0).into();

        vger.translate(self.root_offset);
        self.visible = Some(WorldRect::new(
            WorldPoint::origin(),
            window_size.cast_unit(),
        ));
        view.draw(root, &mut DrawArgs { cx: self, vger });
        self.visible = None;
        self.enable_dirty = true;

        if self.render_dirty {
            let paint = vger.color_paint(RED_HIGHLIGHT);
            let xf = WorldToLocal::identity();
            for rect in self.dirty_region.rects() {
                vger.stroke_rect(
                    xf.transform_point(rect.min()),
                    xf.transform_point(rect.max()),
                    0.0,
                    1.0,
                    paint,
                );
            }
        }

        self.dirty_region.clear();

        let texture_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let desc = wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..<_>::default()
        };

        vger.encode(&desc);

        frame.present();
    }

    /// Process a UI event.
    pub fn process(&mut self, view: &impl View, event: &Event) {
        let mut actions = vec![];
        let root = self.root_id();
        view.process(&event.offset(-self.root_offset), root, self, &mut actions);

        for action in actions {
            if !action.is::<()>() {
                log::debug!("unhandled action: {:?}", (*action).type_id());
            }
        }
    }

    /// Get menu commands.
    pub fn commands(&mut self, view: &impl View, cmds: &mut Vec<CommandInfo>) {
        let root = self.root_id();
        view.commands(root, self, cmds);
    }

    /// Narrows the visible area to `rect`, a rectangle in the local space of `vger`'s current
    /// transform, and returns the area it was before, for `restore_visible`.
    pub(crate) fn clip_visible(&mut self, rect: LocalRect, vger: &Vger) -> Option<WorldRect> {
        let before = self.visible;
        if let Some(visible) = before {
            let world = vger.current_transform().outer_transformed_rect(&rect);
            self.visible = Some(visible.intersection(&world).unwrap_or(WorldRect::zero()));
        }
        before
    }

    /// Restores the visible area that `clip_visible` returned.
    pub(crate) fn restore_visible(&mut self, before: Option<WorldRect>) {
        self.visible = before;
    }

    /// Returns false when `rect`, in the local space of `vger`'s current transform, lies
    /// entirely outside the visible area, so drawing it would show nothing.
    pub(crate) fn is_visible(&self, rect: LocalRect, vger: &Vger) -> bool {
        match self.visible {
            None => true,
            Some(visible) => {
                let world = vger.current_transform().outer_transformed_rect(&rect);
                visible.intersects(&world)
            }
        }
    }

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

    fn node(&self, id: ViewId) -> &Node {
        let node = &self.nodes[id.index as usize];
        assert!(
            node.live && node.generation == id.generation,
            "rui: a view id or state handle refers to a view that no longer exists"
        );
        node
    }

    fn node_mut(&mut self, id: ViewId) -> &mut Node {
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

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
        for id in std::mem::take(&mut self.dirty_nodes) {
            if self.is_live(id) {
                self.nodes[id.index as usize].state_dirty = false;
            }
        }
    }

    pub(crate) fn set_state<S: 'static>(&mut self, id: ViewId, value: S) {
        let node = self.node_mut(id);
        node.state = Some(Box::new(value));
        node.state_dirty = false;
    }

    pub(crate) fn is_dirty(&self, id: ViewId) -> bool {
        self.node(id).state_dirty
    }

    pub(crate) fn init_state<S: 'static, D: Fn() -> S + 'static>(&mut self, id: ViewId, func: &D) {
        let node = self.node_mut(id);
        if node.state.is_none() {
            node.state = Some(Box::new((func)()));
        }
    }

    pub(crate) fn init_env<S: Clone + 'static, D: Fn() -> S + 'static>(&mut self, func: &D) -> S {
        self.env
            .entry(TypeId::of::<S>())
            .or_insert_with(|| Box::new((func)()))
            .downcast_ref::<S>()
            .unwrap()
            .clone()
    }

    pub(crate) fn set_env<S: Clone + 'static>(&mut self, value: &S) -> Option<S> {
        let typeid = TypeId::of::<S>();
        let old_value = self
            .env
            .get(&typeid)
            .map(|b| b.downcast_ref::<S>().unwrap().clone());
        self.env.insert(typeid, Box::new(value.clone()));
        old_value
    }

    pub fn get<S>(&self, id: StateHandle<S>) -> &S
    where
        S: 'static,
    {
        self.node(id.id)
            .state
            .as_ref()
            .expect("rui: no state for handle")
            .downcast_ref()
            .unwrap()
    }

    pub fn get_mut<S>(&mut self, id: StateHandle<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        let node = self.node_mut(id.id);
        let newly_dirty = !node.state_dirty;
        node.state_dirty = true;
        if newly_dirty {
            self.dirty_nodes.push(id.id);
        }
        self.nodes[id.id.index as usize]
            .state
            .as_mut()
            .expect("rui: no state for handle")
            .downcast_mut()
            .unwrap()
    }
}

impl<S> ops::Index<StateHandle<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: StateHandle<S>) -> &S {
        self.get(index)
    }
}

impl<S> ops::IndexMut<StateHandle<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: StateHandle<S>) -> &mut S {
        self.get_mut(index)
    }
}
