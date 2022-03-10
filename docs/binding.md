# Bindings

Bindings allow you to expose parts of your data model to a `View`. For example, you may want to bind a `f32` value in your data model to a slider. The `Binding` trait is defined as follows:

```rust
pub trait Binding<S>: Clone + 'static {
    fn get(&self) -> S;
    fn set(&self, value: S);
}
```

To create a binding for a member of a struct, use the `bind!` macro. Suppose our app state is defined as follows:

```rust
#[derive(Clone)]
struct MyState {
    value: f32,
}
```

then we can use `bind!` to create a control for `value`:

`hslider(bind!(state, value))`

The `bind!` macro simply creates an implementation of the `Binding` trait with the appropriate get/set functions to get and update `value` inside `MyState`.
