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

use tao::window::Window;
pub type CommandMap = HashMap<tao::menu::MenuId, String>;

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

pub trait WindowInterface {
    fn set_title(title: String);
    fn set_fullscreen();
}

pub(crate) type StateMap = HashMap<ViewId, StateHolder>;

pub(crate) type EnvMap = HashMap<TypeId, Box<dyn Any>>;

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

    /// Pressed mouse buton.
    pub(crate) mouse_button: Option<MouseButton>,

    /// Keyboard modifiers state.
    pub key_mods: KeyboardModifiers,

    /// The root view ID. This should be randomized for security reasons.
    root_id: ViewId,

    /// The view that has the keybord focus.
    pub(crate) focused_id: Option<ViewId>,

    /// The tao window
    pub(crate) window: Option<Window>,

    /// The current title of the window
    pub(crate) window_title: String,

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
}

impl Context {
    pub fn new(window: Option<Window>) -> Self {
        Self {
            layout: HashMap::new(),
            touches: [ViewId::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            mouse_button: None,
            key_mods: Default::default(),
            root_id: ViewId { id: 1 },
            focused_id: None,
            window,
            window_title: "rui".into(),
            state_map: HashMap::new(),
            dirty: false,
            enable_dirty: true,
            env: HashMap::new(),
            dirty_region: Region::EMPTY,
        }
    }

    /// Call this after the event queue is cleared.
    pub fn update(
        &mut self,
        view: &impl View,
        vger: &mut Vger,
        access_nodes: &mut Vec<accesskit::Node>,
    ) {
        // Run any animations.
        view.process(&Event::Anim, self.root_id, self, vger);

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
                for node in &nodes {
                    println!(
                        "  id: {:?}, role: {:?}, children: {:?}",
                        node.id, node.role, node.children
                    );
                }
                *access_nodes = nodes;
            } else {
                // println!("access nodes unchanged");
            }

            // XXX: we're doing layout both here and in rendering.
            let window_size = self.window.as_ref().unwrap().inner_size();
            let scale = self.window.as_ref().unwrap().scale_factor() as f32;
            let width = window_size.width as f32 / scale;
            let height = window_size.height as f32 / scale;
            view.layout(self.root_id, [width, height].into(), self, vger);

            // Get dirty rectangles.
            view.dirty(self.root_id, LocalToWorld::identity(), self);

            self.window.as_ref().unwrap().request_redraw();

            self.clear_dirty();
        }
    }

    /// Redraw the UI using wgpu.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        view: &impl View,
        vger: &mut Vger,
    ) {
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                surface.configure(device, config);
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };

        let window_size = self.window.as_ref().unwrap().inner_size();
        let scale = self.window.as_ref().unwrap().scale_factor() as f32;
        // println!("window_size: {:?}", window_size);
        let width = window_size.width as f32 / scale;
        let height = window_size.height as f32 / scale;

        vger.begin(width, height, scale);

        // Disable dirtying the state during layout and rendering
        // to avoid constantly re-rendering if some state is saved.
        self.enable_dirty = false;
        view.layout(self.root_id, [width, height].into(), self, vger);
        view.draw(self.root_id, self, vger);
        self.enable_dirty = true;

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

        self.dirty_region.clear();

        let texture_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let desc = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };

        vger.encode(device, &desc, queue);

        frame.present();
    }

    /// Process a UI event.
    pub fn process(&mut self, view: &impl View, event: &Event, vger: &mut Vger) {
        view.process(event, self.root_id, self, vger);
    }

    /// Get menu commands.
    pub fn commands(&mut self, view: &impl View, cmds: &mut Vec<CommandInfo>) {
        view.commands(self.root_id, self, cmds);
    }

    /// Enter full-screen mode (if available)
    pub fn fullscreen(&mut self) {
        self.window
            .as_ref()
            .unwrap()
            .set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)))
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

    pub fn get<S>(&self, id: State<S>) -> &S
    where
        S: 'static,
    {
        self.state_map[&id.id].state.downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: State<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        let mut holder = self.state_map.get_mut(&id.id).unwrap();
        holder.dirty = true;
        holder.state.downcast_mut::<S>().unwrap()
    }
}

impl<S> ops::Index<State<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: State<S>) -> &Self::Output {
        self.get(index)
    }
}

impl<S> ops::IndexMut<State<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: State<S>) -> &mut Self::Output {
        self.get_mut(index)
    }
}
