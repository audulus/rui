use crate::*;
use accesskit::Role;

pub trait Modifiers: View + Sized {
    /// Adds space around a view. Can be either `Auto` or `Px(number_of_pixels)`
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self>;

    /// Calls a function in response to a tap.
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self, F>;

    /// Puts a view behind another. The background view inherits the size of the view.
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG>;

    /// Calls a function with the view's geometry after layout runs.
    /// Currently only the view's size is returned.
    fn geom<F: Fn(LocalSize) + 'static>(self, f: F) -> Geom<Self, F>;

    /// Calls a function in response to a drag.
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self, F>;

    /// Applies an offset to the view in local space.
    fn offset<Off: Into<LocalOffset>>(self, offset: Off) -> Offset<Self>;

    /// Constrains the size of a view.
    fn size<Sz: Into<LocalSize>>(self, size: Sz) -> Size<Self>;

    /// Adds a menu command.
    fn command<F: Fn() + 'static>(self, name: &str, key: Option<KeyCode>, f: F)
        -> Command<Self, F>;

    /// Adds a group of menu commands.
    fn command_group<T: CommandTuple>(self, cmds: T) -> CommandGroup<Self, T>;

    /// Responds to keyboard events
    fn key<F: Fn(KeyPress) + 'static>(self, f: F) -> Key<Self, F>;

    /// Specify an accessiblity role.
    fn role(self, role: Role) -> RoleView<Self>;

    /// Specify the title of the window.
    fn window_title(self, title: String) -> TitleView<Self>;
}

impl<V: View + 'static> Modifiers for V {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self, F> {
        Tap::new(self, f)
    }
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG> {
        Background::new(self, background)
    }
    fn geom<F: Fn(LocalSize) + 'static>(self, f: F) -> Geom<Self, F> {
        Geom::new(self, f)
    }
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self, F> {
        Drag::new(self, f)
    }
    fn offset<Off: Into<LocalOffset>>(self, offset: Off) -> Offset<Self> {
        Offset::new(self, offset.into())
    }
    fn size<Sz: Into<LocalSize>>(self, size: Sz) -> Size<Self> {
        Size::new(self, size.into())
    }
    fn command<F: Fn() + 'static>(
        self,
        name: &str,
        key: Option<KeyCode>,
        f: F,
    ) -> Command<Self, F> {
        Command::new(self, name.into(), key, f)
    }
    fn command_group<T: CommandTuple>(self, cmds: T) -> CommandGroup<Self, T> {
        CommandGroup::new(self, cmds)
    }
    fn key<F: Fn(KeyPress) + 'static>(self, f: F) -> Key<Self, F> {
        Key::new(self, f)
    }
    fn role(self, role: Role) -> RoleView<Self> {
        RoleView::new(self, role)
    }
    fn window_title(self, title: String) -> TitleView<Self> {
        TitleView::new(self, title)
    }
}
