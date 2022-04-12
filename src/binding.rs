use crate::*;

pub trait Lens<T, U>: Clone + Copy + 'static {
    fn focus<'a>(&self, data: &'a T) -> &'a U;
    fn focus_mut<'a>(&self, data: &'a mut T) -> &'a mut U;
}

/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + Copy + 'static {
    fn get<'a>(&self, cx: &'a mut Context) -> &'a S;
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S;
}

#[derive(Clone)]
pub struct Map<B, L, S, T> {
    binding: B,
    lens: L,
    phantom_s: std::marker::PhantomData<S>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<B, L, S, T> Copy for Map<B, L, S, T>
where
    B: Copy,
    L: Copy,
    S: Clone,
    T: Clone,
{
}

impl<S, B, L, T> Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: Clone + 'static,
    T: Clone + 'static,
{
    pub fn new(binding: B, lens: L) -> Self {
        Self { binding, lens, phantom_s: Default::default(), phantom_t: Default::default() }
    }
}

pub fn bind<S, T>(binding: impl Binding<S>, lens: impl Lens<S, T>) -> impl Binding<T> where S: Clone + 'static, T: Clone + 'static {
    Map::new(binding, lens)
}

impl<S, B, L, T> Binding<S> for Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: Clone + 'static,
    T: Clone + 'static,
{
    fn get<'a>(&self, cx: &'a mut Context) -> &'a S {
        self.lens.focus(self.binding.get(cx))
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        self.lens.focus_mut(self.binding.get_mut(cx))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone)]
    struct MyState {
        x: i32,
    }

    #[derive(Clone, Copy)]
    struct MyLens {}
    impl Lens<MyState, i32> for MyLens {
        fn focus<'a>(&self, data: &'a MyState) -> &'a i32 {
            &data.x
        }
        fn focus_mut<'a>(&self, data: &'a mut MyState) -> &'a mut i32 {
            &mut data.x
        }
    }

    #[test]
    fn test_lens() {
        let mut s = MyState { x: 0 };
        *MyLens {}.focus_mut(&mut s) = 42;
        assert_eq!(*MyLens {}.focus(&s), 42);
    }

    #[test]
    fn test_bind2() {
        let mut cx = Context::new(None);
        let id = ViewID::default();
        cx.state_map
            .entry(id)
            .or_insert_with(|| Box::new(MyState { x: 0 }));
        let s = State::new(id);

        let b = bind(s, MyLens{});

        *b.get_mut(&mut cx) = 42;
    }
}
