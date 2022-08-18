
# rui

Rui is a minimalst declarative UI library for desktop and mobile apps.

With rui, there's very little ceremony or boilerplate to define your UI.
And because your UI is a function of application state, it will automatically
update whenever state changes. No need to manage the updating yourself as in
traditional object-oriented UI libraries.

Underneath rui is vger, a GPU-based vector graphics renderer. Rui will only
re-render a window when your application state changes, and rendering is very
fast. Rui doesn't attempt to use native widgets or emulate them.

Rui is experimental and not yet (or ever) intended for shipping products.

## Key Concepts

- `View` -- your UI is expressed as a tree of views
- `State` -- stores your application state
- `Binding` -- a view into your application state which can be passed to controls
