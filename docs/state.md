# State

`State` holds your app's data model. The `state` function attaches state to a particular position in the view tree. For example:

```rust
`state(0.0, |my_state: State<f32>| {
    slider(my_state)
}
```

`State` implements `Binding`, so it has `get` and `set` functions, and can be passed directly to views.
Typically though, you'd use the `bind!` macro to create a `Binding` to something inside your state, and then pass that to a view.
