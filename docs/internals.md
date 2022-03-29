# Internals

Unlike the classic OOP UI, the `Views` you pass to rui are immutable. Mutable state is stored in the `Context`, which the user never interacts with directly. The state is all keyed by `ViewIDs`.

A `ViewID` is the unique identifier for a view (a u64 internally), formed by hashing a traversal down the view tree.

Methods on the `View` trait are the typical stuff you might see in an OOP API: event processing, rendering, layout. Whenever possible, rui tries to implement views in terms of other views, rather than implementing the methods directly. Where a view needs its own struct type for modifier methods, a `body_view!` macro can be used to fill in `View` trait methods.
