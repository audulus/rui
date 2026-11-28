# gui_bench

The [gui-bench](https://github.com/zortax/gui-bench) scenarios in rui. gui-bench compares Rust GUI
frameworks by running the same app in each one under fakeland, a headless Wayland compositor that
sends real input and times every frame. This is rui's app. The benchmark itself builds the copy
in `apps/rui` of [audulus/gui-bench](https://github.com/audulus/gui-bench), and this one is kept
here so rui changes can be tried against it directly.

## Run

```sh
cd examples/gui_bench
cargo run --release                                     # the workbench app, 100 orders
cargo run --release -- --rows 3000                      # 3,000 orders and messages
cargo run --release -- --scenario list --rows 1000000   # the virtual list
```

The window is 1280x800 without decorations; quit with Cmd+Q or by killing the process.

## Profile

```sh
cargo run --release --example profile_hover                # 100 orders
ROWS=3000 ITERS=5000 cargo run --release --example profile_hover
ONLY=invalidate_all cargo run --release --example profile_hover
```

`examples/profile_hover.rs` runs the `app` scenario headless, with vger's real text measurement
but no window. It times `Context::update` (the anim pass, layout and the dirty walk) with no
writes (`idle`), with the pointer moving between two order rows (`hover`), and with everything
laid out again (`invalidate_all`). `ONLY` picks one case, which is handy under a sampling
profiler. Drawing isn't measured: it needs a window surface.

## What it shows

- **`app`:** a sidebar with nav items, a top bar, metric cards, a bar chart with filters, and a
  table of orders (every row a real view), plus a messages page.
- **`list`:** the orders in a virtual list.

`src/spec/dashboard.rs` fixes every size, color, text and number. The benchmark's probes click,
hover and read pixels at the spec's window coordinates, so the layout has to match it exactly.

## Keeping it in sync

`src/spec/` is vendored from gui-bench's `apps/shared/` and is licensed under the Apache License
2.0 (`LICENSE-APACHE`); the rest of this example is under rui's MIT license. When gui-bench
changes its spec or this app changes, update the other copy too.
