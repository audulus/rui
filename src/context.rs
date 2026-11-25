use crate::*;
use euclid::*;
use std::any::Any;
use std::any::TypeId;
use std::cell::{Cell, RefCell};
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
    pub(crate) nodes: Vec<Node>,

    /// Indices of freed slots in `nodes`.
    pub(crate) free: Vec<u32>,

    /// Counts state writes. Each write takes the next value, so comparing generations tells
    /// whether something was written after a result was computed.
    pub(crate) write_gen: u64,

    /// `write_gen` at the end of the last update: states written later are dirty.
    pub(crate) clean_gen: u64,

    /// Cached results computed before this generation are stale (see `invalidate_all`).
    pub(crate) floor_gen: u64,

    /// During a walk, the latest write generation of the states of the enclosing `state`
    /// views, on which everything under them depends.
    pub(crate) ancestor_gen: u64,

    /// While `state` views compute results to cache (see `begin_reads`), the states read
    /// through `get`, so a cache also goes stale when a state outside its view's ancestors
    /// and subtree is written.
    pub(crate) reads: RefCell<Vec<ViewId>>,

    /// How many `state` views are computing results to cache.
    pub(crate) reading: Cell<u32>,

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
            nodes: vec![Node::new(1, ViewId::default(), 0)],
            free: vec![],
            write_gen: 0,
            clean_gen: 0,
            floor_gen: 0,
            ancestor_gen: 0,
            reads: RefCell::new(vec![]),
            reading: Cell::new(0),
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
            window_size: Size2D::default(),
            root_offset: LocalOffset::zero(),
            visible: None,
            render_dirty: false,
            grab_cursor: false,
            prev_grab_cursor: false,
        }
    }

    /// Call this after the event queue is cleared.
    ///
    /// When anything changed, replaces `access_nodes` with the AccessKit nodes that changed
    /// since the last update, as for an AccessKit `TreeUpdate`: the first update gives the whole
    /// tree, later ones only the nodes under `state` views whose state, or whose enclosing
    /// states, were written.
    pub fn update(
        &mut self,
        view: &impl View,
        vger: &mut Vger,
        access_nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        window_size: Size2D<f32, WorldSpace>,
    ) -> bool {
        // If the window size has changed, force a relayout.
        if window_size != self.window_size {
            self.invalidate_all();
            self.window_size = window_size;
        }

        let root = self.root_id();

        // Run any animations.
        let mut actions = vec![];
        view.process(&Event::Anim, root, self, &mut actions);

        if self.dirty {
            // Get the accesskit nodes that changed.
            access_nodes.clear();
            view.access(root, self, access_nodes);
            log::debug!("access nodes changed: {}", access_nodes.len());

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

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
        self.clean_gen = self.write_gen;
    }

    pub(crate) fn set_state<S: 'static>(&mut self, id: ViewId, value: S) {
        let node = self.node_mut(id);
        node.state = Some(Box::new(value));
        // Setting the value isn't a write by the app: `map` sets it on every walk, and then
        // checks whether its child wrote it.
        node.state_gen = 0;
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

    /// Returns the state that `id` refers to.
    ///
    /// Panics if the view that owns the state no longer exists. A handle can outlive its view
    /// when it is kept in a closure passed to `on_main`; use `try_get` there.
    pub fn get<S>(&self, id: StateHandle<S>) -> &S
    where
        S: 'static,
    {
        if self.reading.get() > 0 {
            self.reads.borrow_mut().push(id.id);
        }
        self.node(id.id)
            .state
            .as_ref()
            .expect("rui: no state for handle")
            .downcast_ref()
            .unwrap()
    }

    /// Returns the state that `id` refers to, for writing, and marks it changed.
    ///
    /// Panics if the view that owns the state no longer exists; see `try_get_mut`.
    pub fn get_mut<S>(&mut self, id: StateHandle<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        self.mark_written(id.id);
        self.nodes[id.id.index as usize]
            .state
            .as_mut()
            .expect("rui: no state for handle")
            .downcast_mut()
            .unwrap()
    }

    /// Returns the state that `id` refers to, or None if the view that owned it no longer
    /// exists (an item removed from a list, say).
    pub fn try_get<S>(&self, id: StateHandle<S>) -> Option<&S>
    where
        S: 'static,
    {
        self.is_live(id.id).then(|| self.get(id))
    }

    /// Like `get_mut`, but returns None if the view that owned the state no longer exists, as
    /// for a background task that finishes after its view went away:
    ///
    /// ```no_run
    /// # use rui::*;
    /// # fn f(s: StateHandle<String>) {
    /// on_main(move |cx| {
    ///     if let Some(text) = cx.try_get_mut(s) {
    ///         *text = "done".into();
    ///     }
    /// });
    /// # }
    /// ```
    pub fn try_get_mut<S>(&mut self, id: StateHandle<S>) -> Option<&mut S>
    where
        S: 'static,
    {
        if self.is_live(id.id) {
            Some(self.get_mut(id))
        } else {
            None
        }
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
