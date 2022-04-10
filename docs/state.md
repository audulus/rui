# State

`State` holds your app's data model. The `state` function attaches state to a particular position ([`ViewID`](internals.md)) in the view tree. For example:

```rust
state(0.0, |my_state: State<f32>| {
    slider(my_state)
})
```

`State` implements [`Binding`](binding.md), so it has `get` and `set` functions, and can be passed directly to views.
Typically though, you'd use the `bind!` macro to create a `Binding` to something inside your state, and then pass that to a view.

The type held by `State` must implement `Clone`, and do so inexpensively. Use `Arc` as necessary to avoid cloning
pieces of your data model. Consider using [immutable data structures](https://crates.io/crates/im) for your data model.

`State` uses `Arc/Mutex` internally and can be used from any thread. See [`examples/async.rs`](../examples/async.rs).
