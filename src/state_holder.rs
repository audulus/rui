use std::any::Any;

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}