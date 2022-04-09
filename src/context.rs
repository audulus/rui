use crate::*;
use euclid::*;
use std::any::Any;
use std::collections::HashMap;

pub type LocalSpace = vger::defs::LocalSpace;
pub type WorldSpace = vger::defs::WorldSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;
pub type WorldPoint = Point2D<f32, WorldSpace>;

use tao::event_loop::EventLoopProxy;
use tao::window::Window;

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

/// Restricts what we can store in a StateMap (instead of just using Any)
pub trait AnyState {
    /// So we can downcast.
    fn as_any(&self) -> &dyn Any;
}

pub type StateMap = HashMap<ViewID, Box<dyn AnyState>>;

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Map for `state`.
    pub(crate) state_map: StateMap,

    /// Layout information for all views.
    pub(crate) layout: HashMap<ViewID, LayoutBox>,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewID; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [LocalPoint; 16],

    /// The root view ID. This should be randomized for security reasons.
    pub(crate) root_id: ViewID,

    /// The view that has the keybord focus.
    pub(crate) focused_id: Option<ViewID>,

    /// The tao window
    pub(crate) window: Window,

    /// The current title of the window
    pub(crate) window_title: String,

    /// Allows us to wake up the event loop.
    pub(crate) event_loop_proxy: Option<EventLoopProxy<()>>,
}

impl Context {
    pub fn new(event_loop_proxy: Option<EventLoopProxy<()>>, window: Window) -> Self {
        Self {
            state_map: HashMap::new(),
            layout: HashMap::new(),
            touches: [ViewID::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            root_id: ViewID { id: 1 },
            focused_id: None,
            window,
            window_title: "rui".into(),
            event_loop_proxy,
        }
    }

    pub fn get_state<S: Clone + 'static, D: Fn() -> S>(&mut self, id: ViewID, default: &D) -> State<S> {
        let proxy = self.event_loop_proxy.clone();
        let s = self
            .state_map
            .entry(id)
            .or_insert_with(|| Box::new(State::new(default(), proxy)));

        if let Some(state) = s.as_any().downcast_ref::<State<S>>() {
            state.clone()
        } else {
            panic!("state has wrong type")
        }
    }

}
