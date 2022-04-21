use crate::*;

/// Allows rui to iterate over a tuple of `Views`.
pub trait ViewTuple {
    fn foreach_view<F: FnMut(&dyn View)>(&self, f: &mut F);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        false
    } // satisfy clippy
}

macro_rules! impl_view_tuple {
    ( $n: tt; $( $t:ident),* ; $( $s:tt ),* ) => {

        impl< $( $t: View, )* > ViewTuple for ( $( $t, )* ) {
            fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
                $( f(&self.$s); )*
            }
            fn len(&self) -> usize {
                $n
            }
        }
    }
}

pub const VIEW_TUPLE_MAX_ELEMENTS: usize = 10;

impl_view_tuple!(1; V0; 0);
impl_view_tuple!(2; V0, V1; 0, 1);
impl_view_tuple!(3; V0, V1, V2; 0, 1, 2);
impl_view_tuple!(4; V0, V1, V2, V3; 0, 1, 2, 3);
impl_view_tuple!(5; V0, V1, V2, V3, V4; 0, 1, 2, 3, 4);
impl_view_tuple!(6; V0, V1, V2, V3, V4, V5; 0, 1, 2, 3, 4, 5);
impl_view_tuple!(7; V0, V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5, 6);
impl_view_tuple!(8;
    V0, V1, V2, V3, V4, V5, V6, V7;
    0, 1, 2, 3, 4, 5, 6, 7
);
impl_view_tuple!(9;
    V0, V1, V2, V3, V4, V5, V6, V7, V8;
    0, 1, 2, 3, 4, 5, 6, 7, 8
);
impl_view_tuple!(10;
    V0, V1, V2, V3, V4, V5, V6, V7, V8, V9;
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9
);
