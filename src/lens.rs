pub trait Lens<T, U>: Clone + Copy + 'static {
    fn focus<'a>(&self, data: &'a T) -> &'a U;
    fn focus_mut<'a>(&self, data: &'a mut T) -> &'a mut U;
}

#[macro_export]
macro_rules! make_lens {
    ($lens_name: ident, $from: ty, $to: ty, $field: ident) => {
        #[derive(Clone, Copy)]
        struct $lens_name {}
        impl Lens<$from, $to> for $lens_name {
            fn focus<'a>(&self, data: &'a $from) -> &'a $to {
                &data.$field
            }
            fn focus_mut<'a>(&self, data: &'a mut $from) -> &'a mut $to {
                &mut data.$field
            }
        }
    };
}
