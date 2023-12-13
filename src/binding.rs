use crate::*;

/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + Copy + 'static {
    fn get<'a>(&self, cx: &'a Context) -> &'a S;
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S;

    fn with<T>(&self, cx: &Context, f: impl FnOnce(&S) -> T) -> T {
        f(self.get(cx))
    }

    fn with_mut<T>(&self, cx: &mut Context, f: impl FnOnce(&mut S) -> T) -> T {
        f(self.get_mut(cx))
    }
}

pub fn setter<S>(binding: impl Binding<S>) -> impl Fn(S, &mut Context) {
    move |s, cx| binding.with_mut(cx, |v| *v = s)
}

pub struct Map<B, L, S, T> {
    binding: B,
    lens: L,
    phantom_s: std::marker::PhantomData<S>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<B, L, S, T> Clone for Map<B, L, S, T>
where
    B: Copy,
    L: Copy,
    S: 'static,
    T: 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, L, S, T> Copy for Map<B, L, S, T>
where
    B: Copy,
    L: Copy,
    S: 'static,
    T: 'static,
{
}

impl<S, B, L, T> Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: 'static,
    T: 'static,
{
    pub fn new(binding: B, lens: L) -> Self {
        Self {
            binding,
            lens,
            phantom_s: Default::default(),
            phantom_t: Default::default(),
        }
    }
}

pub fn bind<S, T>(binding: impl Binding<S>, lens: impl Lens<S, T>) -> impl Binding<T>
where
    S: 'static,
    T: 'static,
{
    Map::new(binding, lens)
}

impl<S, B, L, T> Binding<S> for Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: 'static,
    T: 'static,
{
    fn get<'a>(&self, cx: &'a Context) -> &'a S {
        self.lens.focus(self.binding.get(cx))
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        self.lens.focus_mut(self.binding.get_mut(cx))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone, Default)]
    struct MyState {
        x: i32,
    }

    make_lens!(MyLens, MyState, i32, x);

    #[test]
    fn test_lens() {
        let mut s = MyState { x: 0 };
        *MyLens {}.focus_mut(&mut s) = 42;
        assert_eq!(*MyLens {}.focus(&s), 42);
    }

    #[test]
    fn test_bind() {
        let mut cx = Context::new();
        let id = ViewId::default();
        cx.init_state(id, &MyState::default);
        let s = StateHandle::new(id, &mut cx);

        let b = bind(s, MyLens {});

        *b.get_mut(&mut cx) = 42;

        assert_eq!(*b.get(&cx), 42);
    }
}
