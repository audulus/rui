use crate::*;
use accesskit::Role;
use std::marker::PhantomData;

/// Modifiers common to all views.
pub trait Modifiers: View + Sized {
    /// Calls a closure after rendering with context and delta time.
    fn anim<F: Fn(&mut Context, f32) + 'static + Clone>(self, func: F) -> AnimView<Self, F> {
        AnimView::new(self, func)
    }

    /// Puts a view behind another. The background view inherits the size of the view.
    fn background<BG: View + Clone>(self, background: BG) -> Background<Self, BG> {
        Background::new(self, background)
    }

    /// Adds a menu command.
    fn command<F: Fn(&mut Context) + Clone + 'static>(
        self,
        name: &str,
        key: Option<HotKey>,
        f: F,
    ) -> Command<Self, F> {
        Command::new(self, name.into(), key, f)
    }

    /// Adds a group of menu commands.
    fn command_group<T: CommandTuple>(self, cmds: T) -> CommandGroup<Self, T> {
        CommandGroup::new(self, cmds)
    }

    /// Calls a function in response to a drag.
    fn drag<
        F: Fn(&mut Context, LocalOffset, GestureState, Option<MouseButton>) + Clone + 'static,
    >(
        self,
        f: F,
    ) -> Drag<Self, DragFunc<F>> {
        Drag::new(self, DragFunc { f })
    }

    /// Calls a function in response to a drag. Version which passes the position.
    fn drag_p<
        F: Fn(&mut Context, LocalPoint, GestureState, Option<MouseButton>) + Clone + 'static,
    >(
        self,
        f: F,
    ) -> Drag<Self, DragFuncP<F>> {
        Drag::new(self, DragFuncP { f })
    }

    /// Calls a function in response to a drag. Version which passes in a binding.
    fn drag_s<
        T: Clone + 'static,
        B: Binding<T>,
        F: Fn(&mut T, LocalOffset, GestureState, Option<MouseButton>) + Clone + 'static,
    >(
        self,
        b: B,
        f: F,
    ) -> Drag<Self, DragFuncS<F, B, T>> {
        Drag::new(
            self,
            DragFuncS {
                f,
                b,
                phantom: PhantomData::default(),
            },
        )
    }

    /// Calls a function in response to a mouse hovering.
    fn hover<A: 'static, F: Fn(&mut Context, bool) -> A + Clone + 'static>(
        self,
        f: F,
    ) -> Hover<Self, HoverFunc<F>> {
        Hover::new(self, HoverFunc { f })
    }

    /// Calls a function in response to a mouse hovering. Version which passes the position
    fn hover_p<A: 'static, F: Fn(&mut Context, LocalPoint) -> A + Clone + 'static>(
        self,
        f: F,
    ) -> Hover<Self, HoverFuncP<F>> {
        Hover::new(self, HoverFuncP { f })
    }

    /// Add an environment value.
    fn env<E: Clone + 'static>(self, value: E) -> SetenvView<Self, E> {
        SetenvView::new(self, value)
    }

    /// Indicates that this item can expand within a stack.
    fn flex(self) -> Flex<Self> {
        Flex::new(self)
    }

    /// Make the window full screen.
    fn fullscreen(self) -> FullscreenView<Self> {
        FullscreenView::new(self)
    }

    /// Calls a function with the view's geometry after layout runs.
    /// Currently only the view's size is returned.
    fn geom<F: Fn(&mut Context, LocalSize, LocalToWorld) + Clone + 'static>(
        self,
        f: F,
    ) -> Geom<Self, F> {
        Geom::new(self, f)
    }

    /// Responds to keyboard events
    fn key<F: Fn(&mut Context, Key) + Clone + 'static>(self, f: F) -> KeyView<Self, F> {
        KeyView::new_pressed(self, f)
    }

    /// Responds to keyboard events
    fn key_released<F: Fn(&mut Context, Key) + Clone + 'static>(self, f: F) -> KeyView<Self, F> {
        KeyView::new_released(self, f)
    }

    /// Applies an offset to the view in local space.
    fn offset<Off: Into<LocalOffset>>(self, offset: Off) -> Offset<Self> {
        Offset::new(self, offset.into())
    }

    /// Adds space around a view. Can be either `Auto` or `Px(number_of_pixels)`
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }

    /// Specify an accessiblity role.
    fn role(self, role: Role) -> RoleView<Self> {
        RoleView::new(self, role)
    }

    /// Constrains the size of a view.
    fn size<Sz: Into<LocalSize>>(self, size: Sz) -> Size<Self> {
        Size::new(self, size.into())
    }

    /// Calls a function in response to a tap.
    fn tap<A: 'static, F: Fn(&mut Context) -> A + Clone + 'static>(
        self,
        f: F,
    ) -> Tap<Self, TapAdapter<F>> {
        Tap::new(self, TapAdapter { f })
    }

    /// Version of `tap` which takes an action type instead
    /// of a function.
    fn tap_a<A: Clone + 'static>(self, action: A) -> Tap<Self, TapActionAdapter<A>> {
        Tap::new(self, TapActionAdapter { action })
    }

    /// Version of `tap` which passes the tap position and mouse button.
    fn tap_p<
        A: 'static,
        F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + Clone + 'static,
    >(
        self,
        f: F,
    ) -> Tap<Self, TapPositionFunc<F>> {
        Tap::new(self, TapPositionFunc { f })
    }

    /// Calls a function in response to a touch.
    /// #### Why use this?
    /// * You need to know the position of the touch.
    /// * You need to handle the beginning and end of the touch.
    ///
    /// #### Example
    /// ```rust
    /// use rui::*;
    /// rectangle()
    ///     .touch(move |_, info| match info.state {
    ///         TouchState::Begin => { println!("Touched") }
    ///         TouchState::End => { println!("Released") }
    ///     });
    ///     //.run();
    /// ```
    fn touch<A: 'static, F: Fn(&mut Context, TouchInfo) -> A + Clone + 'static>(
        self,
        f: F,
    ) -> Touch<Self, TouchFunc<F>> {
        Touch::new(self, TouchFunc { f })
    }

    /// Specify the title of the window.
    fn window_title(self, title: &str) -> TitleView<Self> {
        TitleView::new(self, title)
    }

    /// Handle an action from a child view.
    fn handle<A: 'static, A2: 'static, F: Fn(&mut Context, &A) -> A2 + Clone + 'static>(
        self,
        handler: F,
    ) -> Handle<Self, F, A, A2> {
        Handle::new(self, handler)
    }

    /// Clip to bounds.
    fn clip(self) -> Clip<Self> {
        Clip::new(self)
    }
}

impl<V: View> Modifiers for V {}
