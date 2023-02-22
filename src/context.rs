use crate::*;
use euclid::*;
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops;

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
    pub path: String,
    pub key: Option<HotKey>,
}

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}

pub(crate) type StateMap = HashMap<ViewId, StateHolder>;

pub(crate) type EnvMap = HashMap<TypeId, Box<dyn Any>>;

pub struct RenderInfo<'a> {
    pub device: &'a wgpu::Device,
    pub surface: &'a wgpu::Surface,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub queue: &'a wgpu::Queue,
}

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Layout information for all views.
    pub(crate) layout: HashMap<ViewId, LayoutBox>,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewId; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [LocalPoint; 16],

    /// Pressed mouse button.
    pub(crate) mouse_button: Option<MouseButton>,

    /// Keyboard modifiers state.
    pub key_mods: KeyboardModifiers,

    /// The root view ID. This should be randomized for security reasons.
    pub(crate) root_id: ViewId,

    /// The view that has the keyboard focus.
    pub(crate) focused_id: Option<ViewId>,

    /// The current title of the window
    pub window_title: String,

    /// Are we fullscreen?
    pub fullscreen: bool,

    /// User state created by `state`.
    pub(crate) state_map: StateMap,

    /// Has the state changed?
    pub(crate) dirty: bool,

    /// Are we currently setting the dirty bit?
    pub(crate) enable_dirty: bool,

    /// Values indexed by type.
    pub(crate) env: EnvMap,

    /// Regions of window that needs repainting.
    pub(crate) dirty_region: Region<WorldSpace>,

    /// State dependencies.
    pub(crate) deps: HashMap<ViewId, Vec<ViewId>>,

    /// A stack of ids for states to get parent dependencies.
    pub(crate) id_stack: Vec<ViewId>,

    /// Previous window size.
    window_size: Size2D<f32, WorldSpace>,

    /// Offset for events at the root level.
    root_offset: LocalOffset,

    /// Render the dirty rectangle for debugging?
    render_dirty: bool,

    pub(crate) access_node_classes: accesskit::NodeClassSet,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            layout: HashMap::new(),
            touches: [ViewId::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            mouse_button: None,
            key_mods: Default::default(),
            root_id: ViewId { id: 1 },
            focused_id: None,
            window_title: "rui".into(),
            fullscreen: false,
            state_map: HashMap::new(),
            dirty: false,
            enable_dirty: true,
            env: HashMap::new(),
            dirty_region: Region::EMPTY,
            deps: HashMap::new(),
            id_stack: vec![],
            window_size: Size2D::default(),
            root_offset: LocalOffset::zero(),
            render_dirty: false,
            access_node_classes: accesskit::NodeClassSet::default(),
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
            self.deps.clear();
            self.window_size = window_size;
        }

        // Run any animations.
        let mut actions = vec![];
        view.process(&Event::Anim, self.root_id, self, &mut actions);

        if self.dirty {
            // Clean up state.
            let mut keep = vec![];
            view.gc(self.root_id, self, &mut keep);
            let keep_set = HashSet::<ViewId>::from_iter(keep);
            self.state_map.retain(|k, _| keep_set.contains(k));

            // Get a new accesskit tree.
            let mut nodes = vec![];
            view.access(self.root_id, self, &mut nodes);

            if nodes != *access_nodes {
                println!("access nodes:");
                for (id, node) in &nodes {
                    println!(
                        "  id: {:?} role: {:?}, children: {:?}",
                        id,
                        node.role(),
                        node.children()
                    );
                }
                *access_nodes = nodes;
            } else {
                // println!("access nodes unchanged");
            }

            // XXX: we're doing layout both here and in rendering.
            view.layout(
                self.root_id,
                &mut LayoutArgs {
                    sz: [window_size.width, window_size.height].into(),
                    cx: self,
                    text_bounds: &mut |str, size, max_width| vger.text_bounds(str, size, max_width),
                },
            );

            // Get dirty rectangles.
            view.dirty(self.root_id, LocalToWorld::identity(), self);

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

        // Disable dirtying the state during layout and rendering
        // to avoid constantly re-rendering if some state is saved.
        self.enable_dirty = false;
        let local_window_size = window_size.cast_unit::<LocalSpace>();
        let sz = view.layout(
            self.root_id,
            &mut LayoutArgs {
                sz: local_window_size,
                cx: self,
                text_bounds: &mut |str, size, max_width| vger.text_bounds(str, size, max_width),
            },
        );

        // Center the root view in the window.
        self.root_offset = ((local_window_size - sz) / 2.0).into();

        vger.translate(self.root_offset);
        view.draw(self.root_id, &mut DrawArgs { cx: self, vger });
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
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        vger.encode(device, &desc, render_info.queue);

        frame.present();
    }

    /// Process a UI event.
    pub fn process(&mut self, view: &impl View, event: &Event) {
        let mut actions = vec![];
        view.process(
            &event.offset(-self.root_offset),
            self.root_id,
            self,
            &mut actions,
        );

        for action in actions {
            if !action.is::<()>() {
                println!("unhandled action: {:?}", action.type_id());
            }
        }
    }

    /// Get menu commands.
    pub fn commands(&mut self, view: &impl View, cmds: &mut Vec<CommandInfo>) {
        view.commands(self.root_id, self, cmds);
    }

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
        for holder in &mut self.state_map.values_mut() {
            holder.dirty = false;
        }
    }

    pub(crate) fn set_state<S: 'static>(&mut self, id: ViewId, value: S) {
        self.state_map.insert(
            id,
            StateHolder {
                state: Box::new(value),
                dirty: false,
            },
        );
    }

    pub(crate) fn is_dirty(&self, id: ViewId) -> bool {
        self.state_map[&id].dirty
    }

    pub(crate) fn init_state<S: 'static, D: Fn() -> S + 'static>(&mut self, id: ViewId, func: &D) {
        self.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((func)()),
            dirty: false,
        });
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
        self.state_map[&id.id].state.downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: StateHandle<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        let mut holder = self.state_map.get_mut(&id.id).unwrap();
        holder.dirty = true;
        holder.state.downcast_mut::<S>().unwrap()
    }
}

impl<S> ops::Index<StateHandle<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: StateHandle<S>) -> &Self::Output {
        self.get(index)
    }
}

impl<S> ops::IndexMut<StateHandle<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: StateHandle<S>) -> &mut Self::Output {
        self.get_mut(index)
    }
}
