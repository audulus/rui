/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + 'static {
    fn with<T, F: Fn(&S) -> T>(&self, f: F) -> T;
    fn with_mut<T, F: Fn(&mut S) -> T>(&self, f: F) -> T;

    fn get(&self) -> S
    where
        S: Clone,
    {
        self.with(|s| s.clone())
    }

    fn set(&self, value: S)
    where
        S: Clone,
    {
        self.with_mut(move |s| *s = value.clone());
    }
}

#[derive(Clone)]
pub struct Map<Get, Set> {
    pub getf: Get,
    pub setf: Set,
}

impl<S, Get, Set> Binding<S> for Map<Get, Set>
where
    Get: Fn() -> S + Clone + 'static,
    Set: Fn(S) + Clone + 'static,
    S: Clone,
{
    fn with<T, F: Fn(&S) -> T>(&self, f: F) -> T {
        let v = (self.getf)();
        f(&v)
    }
    fn with_mut<T, F: Fn(&mut S) -> T>(&self, f: F) -> T {
        let mut v = (self.getf)();
        let t = f(&mut v);
        (self.setf)(v);
        t
    }
}

/// Constructs a new binding from a binding and an expression.
/// 
/// For example `bind(b, x)` will create a binding to
/// a member x inside b.
/// 
/// `bind(b, [i])` will create a binding to the ith array
/// element in b.
/// 
/// `bind(b, x[i])` will create a binding to the ith array
/// element of member x in b.
#[macro_export]
macro_rules! bind {
    ( $state:expr, $field:ident ) => {{
        let sref = &$state;
        let state1 = sref.clone();
        let state2 = sref.clone();
        Map {
            getf: move || state1.get().$field.clone(),
            setf: move |val| {
                state2.with_mut(|v| v.$field = val);
            },
        }
    }};
    ( $state:expr, $field:ident [$index:expr] ) => {{
        let sref = &$state;
        let state1 = sref.clone();
        let state2 = sref.clone();
        let idx = $index;
        Map {
            getf: move || state1.get().$field[idx].clone(),
            setf: move |val| {
                state2.with_mut(|v| v.$field[idx] = val);
            },
        }
    }};
    ( $state:expr, [$index:expr] ) => {{
        let sref = &$state;
        let state1 = sref.clone();
        let state2 = sref.clone();
        let idx = $index;
        Map {
            getf: move || state1.get()[idx].clone(),
            setf: move |val| {
                let mut s = state2.get();
                s[idx] = val;
                state2.set(s);
            },
        }
    }};
}

pub fn bind<S, Get, Set>(getf: Get, setf: Set) -> impl Binding<S>
where
    Get: Fn() -> S + Clone + 'static,
    Set: Fn(S) + Clone + 'static,
    S: Clone,
{
    Map { getf, setf }
}

/// Similar to `bind!` but avoids cloning. Requres both the type
/// of the binding and the type of the field to be passed in.
/// 
/// For example: `bind_no_clone!(state, MyState, value, f32)`
#[macro_export]
macro_rules! bind_no_clone {
    ( $state:expr, $type:ident, $field:ident, $type2:ident ) => {{
        #[derive(Clone)]
        struct Bnd<B> {
            binding: B,
        }

        impl<B> Binding<$type2> for Bnd<B>
        where
            B: Binding<$type>,
        {
            fn with<T, F: Fn(&$type2) -> T>(&self, f: F) -> T {
                self.binding.with(|v| f(&v.$field))
            }
            fn with_mut<T, F: Fn(&mut $type2) -> T>(&self, f: F) -> T {
                self.binding.with_mut(|v| f(&mut v.$field))
            }
        }

        Bnd { binding: $state }
    }};
}

// WIP. Attempting again to creating nice bindings without mutation.
#[derive(Clone)]
struct Bnd2<B, L, T1> {
    binding: B,
    lens: L,
    phantom: std::marker::PhantomData<T1>,
}

pub trait Lens<T: ?Sized, U: ?Sized> {
    fn with<V, F: FnOnce(&U) -> V>(&self, data: &T, f: F) -> V;
    fn with_mut<V, F: FnOnce(&mut U) -> V>(&self, data: &mut T, f: F) -> V;
}

impl<B, T0, T1, L> Binding<T0> for Bnd2<B, L, T1>
where
    B: Binding<T1>,
    L: Lens<T1, T0> + Clone + 'static,
    T1: Clone + 'static
{
    fn with<T, F: Fn(&T0) -> T>(&self, f: F) -> T {
        self.binding.with(|v| self.lens.with(v, |vv| f(vv)))
    }
    fn with_mut<T, F: Fn(&mut T0) -> T>(&self, f: F) -> T {
        self.binding.with_mut(|v| self.lens.with_mut(v, |vv| f(vv)))
    }
}

