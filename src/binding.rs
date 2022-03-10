
/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + 'static {
    fn get(&self) -> S;
    fn set(&self, value: S);
    fn with<T, F: Fn(&S) -> T>(&self, f: F) -> T;
    fn with_mut<T, F: Fn(&mut S) -> T>(&self, f: F) -> T;
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
{
    fn get(&self) -> S {
        (self.getf)()
    }
    fn set(&self, value: S) {
        (self.setf)(value);
    }
    fn with<T, F: Fn(&S) -> T>(&self, f: F) -> T {
        let v = self.get();
        f(&v)
    }
    fn with_mut<T, F: Fn(&mut S) -> T>(&self, f: F) -> T {
        let mut v = self.get();
        let t = f(&mut v);
        self.set(v);
        t
    }
}

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
    }}
}

pub fn bind<S, Get, Set>(getf: Get, setf: Set) -> impl Binding<S>
   where Get: Fn() -> S + Clone + 'static, Set: Fn(S) + Clone + 'static {
       Map { getf, setf }
}

#[macro_export]
macro_rules! bind2 {
    ( $state:expr, $type:ident, $field:ident, $type2:ident ) => {{

        #[derive(Clone)]
        struct Bnd<B> {
            binding: B,
        }

        impl <B> Binding<$type2> for Bnd<B> where B:Binding<$type> {
            fn get(&self) -> $type2 {
                self.binding.get().$field.clone()
            }
            fn set(&self, value: $type2) {
                let mut v = self.binding.get();
                v.$field = value;
                self.binding.set(v);
            }
            fn with<T, F: Fn(&$type2) -> T>(&self, f: F) -> T {
                self.binding.with(|v| {
                    f(&v.$field)
                })
            }
            fn with_mut<T, F: Fn(&mut $type2) -> T>(&self, f: F) -> T {
                self.binding.with_mut(|v| {
                    f(&mut v.$field)
                })
            }
        }

        Bnd { binding: $state }
    }};
}