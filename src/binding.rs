
/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + Copy + 'static {
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T;
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T;

    fn get(&self) -> S
    where
        S: Clone,
    {
        self.with(|s| s.clone())
    }

    fn set(&self, value: S) {
        self.with_mut(move |s| *s = value);
    }
}

#[derive(Clone, Copy)]
pub struct Map<Get, Set> {
    pub getf: Get,
    pub setf: Set,
}

impl<S, Get, Set> Binding<S> for Map<Get, Set>
where
    Get: Fn() -> S + Clone + Copy + 'static,
    Set: Fn(S) + Clone + Copy + 'static,
    S: Clone,
{
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T {
        let v = (self.getf)();
        f(&v)
    }
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T {
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
            getf: move || state1.with(|v| v.$field.clone()),
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
            getf: move || state1.with(|v| v.$field[idx].clone()),
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
            getf: move || state1.with(|v| v[idx].clone()),
            setf: move |val| {
                state2.with_mut(|v| v[idx] = val);
            },
        }
    }};
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::*;

//     #[derive(Clone)]
//     struct BindingTestData {
//         x: usize,
//     }

//     #[test]
//     fn test_bind() {
//         let s = State::new(BindingTestData { x: 0 }, None);
//         let b = bind!(s, x);
//         b.set(42);
//         assert_eq!(s.get().x, 42);
//     }
// }