use std::any::Any;
use std::sync::Mutex;

use crate::*;

lazy_static! {
    /// Blasphemy!
    static ref GLOBAL_STATE_MAP: Mutex<HashMap<ViewID, Box<dyn Any + Send>>> = Mutex::new(HashMap::new());
}

/// Contains application state.
#[derive(Clone, Copy)]
pub struct GState<S> {
     id: ViewID,
     phantom: std::marker::PhantomData<S>,
}

impl<S> GState<S> 
where 
    S: Send + 'static
{
    pub fn new(value: S, id: ViewID) -> Self {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        map.insert(id, Box::new(value));
        Self {
            id,
            phantom: std::marker::PhantomData::default()
        }
    }
}

impl<S> Binding<S> for GState<S>
where
    S: Clone + Send + Default + 'static,
{
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        let s = map.entry(self.id)
                   .or_insert_with(|| Box::new(S::default()));
        if let Some(state) = s.downcast_ref::<S>() {
            f(&state)
        } else {
            panic!("state has wrong type")
        }
    }
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        let s = map.entry(self.id)
                       .or_insert_with(|| Box::new(S::default()));
        if let Some(mut state) = s.downcast_mut::<S>() {
            f(&mut state)
        } else {
            panic!("state has wrong type")
        }
    }
}
