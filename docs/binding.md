# Bindings

Bindings allow you to expose parts of your data model to a `View`. For example, you may want to bind a `f32` value in your data model to a slider. The `Binding` trait is defined as follows:

```rust
pub trait Binding<S>: Clone + 'static {
    fn get<'a>(&self, cx: &'a mut Context) -> &'a S;
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S;
}
```

To create a binding for a member of a struct, use `make_lens!` and `bind` Suppose our app state is defined as follows:

```rust
struct MyState {
    value: f32,
}
make_lens!(MyLens, MyState, f32, x);
```

then we can use `bind` to create a control for `value`:

`hslider(bind(state, MyLens{}))`

