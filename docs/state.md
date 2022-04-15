# State

`State` holds your app's data model. The `state` function attaches state to a particular position ([`ViewId`](internals.md)) in the view tree. For example:

```rust
state(0.0, |my_state: State<f32>| {
    slider(my_state)
})
```

`State` implements [`Binding`](binding.md), so it has `get` and `get_mut` functions, and can be passed directly to views.
Typically though, you'd use `bind` to create a `Binding` to something inside your state, and then pass that to a view.

`State` can be passed to other threads, but since the `Context` cannot be passed to other threads, you can't actually update values from a background thread. Instead, use `on_main`. See [`examples/async.rs`](../examples/async.rs).
