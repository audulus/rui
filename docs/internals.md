# Internals

Unlike the classic OOP UI, the `Views` you pass to rui are immutable. They are rebuilt from your functions whenever they're needed and never diffed. Mutable state is stored in the `Context`.

## The node tree

The `Context` keeps a tree of nodes that mirrors the views, stored in an arena. A node holds what one view needs to keep between frames: its layout box, the value of its `state`, and cached results. A view's node is identified by a `ViewId`: an index into the arena plus a generation.

Every method of a view gets the `ViewId` of its node, and passes child `i` the id `cx.child(vid, i)`, which creates the child's node the first time. So a node is identified by its position in the tree, the same way each time the views are rebuilt. The items of a `list` are matched to their nodes by key instead, so an item keeps its state when the list is sorted or filtered.

A node is freed when its view goes away: an item whose key is no longer in its list, the branch a `cond` no longer shows, or the child of an `any_view` that now holds another type of view. Freeing changes the slot's generation, so a `StateHandle` that outlives its view (in a closure passed to `on_main`, say) is detected instead of reading the state of whatever reused the slot. `Context::try_get_mut` returns None for such a handle.

## Skipping what didn't change

Each state write takes the next value of a counter, the write generation, which the state's node records, along with all its ancestors. A result computed at some generation is still valid if nothing at or under its node, and no enclosing `state`, was written since. A view can also read other states, such as a handle kept in the model: while a `state` view computes a result to cache, the `Context` records the states read, and keeps those outside the view's ancestors and subtree with the result, which goes stale when one of them is written. Walks over the tree use that to skip what didn't change:

- A `state` view keeps its last layout, and returns it without calling its function while it is valid and the proposed size is the same. A `list` checks each item's node the same way before building the item's view, so a change to one item lays out only that item.
- The accessibility pass skips `state` views whose nodes haven't changed; `Context::update` reports only the AccessKit nodes that changed.
- The pass that finds the regions to repaint skips subtrees with no writes since the last update.
- The `Event::Anim` pass skips `state` views with no `anim` view under them.

Changes that aren't state writes, such as a new window size or keyboard focus, make every cached result stale. An app that changes something else views read while building, such as a value in a `RefCell`, calls `Context::invalidate_all`.

Methods on the `View` trait are the typical stuff you might see in an OOP API: event processing, rendering, layout. Whenever possible, rui tries to implement views in terms of other views, rather than implementing the methods directly. See `examples/custom_modifier.rs` to add modifiers to a view defined through composition.
