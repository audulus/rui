# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rui is an experimental declarative UI library for Rust, inspired by SwiftUI. It focuses on GPU-rendered UIs that update reactively when state changes, without a retained view tree or DOM diffing. Views are rebuilt from the app's functions and never diffed; per-view data lives in a tree of nodes in the `Context`, which lets walks skip the parts of the UI whose state didn't change.

## Common Development Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo check` - Check for compilation errors without building

### Running Examples
- `cargo run --example <example_name>` - Run a specific example (e.g., `counter`, `shapes`, `canvas`, `slider`, `gallery`)
- `cd examples/<example_name> && cargo run` - For examples with their own Cargo.toml (calculator, synth, flip_cards)

Key examples:
- `cargo run --example counter` - Basic counter demo
- `cargo run --example gallery` - Widget gallery showing all components
- `cargo run --example shapes` - Basic shapes rendering
- `cargo run --example canvas` - GPU vector graphics with vger
- `cd examples/calculator && cargo run` - Calculator app

### Testing Examples
Examples serve as both demos and integration tests. Test by running them and verifying they work correctly.

## Architecture Overview

### Core Components

**View System**: The fundamental building block is the `View` trait, which represents immutable UI components. Views are composed hierarchically to build complex UIs.

**Context**: Stores all mutable UI state in a tree of nodes, one per view, kept in an arena (`src/node.rs`). The `Context` manages:
- Layout information for all views
- User state created by `state` functions
- Cached layout and accessibility results
- Touch/mouse interaction state
- Environment values
- Write generations for reactive updates

**ViewId System**: A `ViewId` is a view's node: an index into the arena plus a generation. Every `DynView` method gets its view's `ViewId` and passes child `i` the id `cx.child(vid, i)`, which creates the node the first time. List items are matched to nodes by key (`sync_keyed_children`), the hidden branch of a `cond` is freed (`sole_child`), and an `any_view` child is freed when its type changes (`typed_child`). Freeing changes the slot's generation, so stale ids and `StateHandle`s are detected.

**Reactive Updates**: Each state write takes the next write generation, recorded on the state's node and all its ancestors. A `state` view reuses its cached layout and accessibility result while nothing under it, no enclosing state, and no other state it read (recorded by `Context::get` between `begin_reads` and `end_reads`) was written since; a `list` checks an item's node before building its view. Changes that aren't state writes (window size, focus) call `Context::invalidate_all`. Multiple state changes in a single event cycle are coalesced.

### Key Concepts

**Views**: Immutable components that define UI structure. Examples: `text()`, `button()`, `vstack()`, `hstack()`, `canvas()`

**State**: Managed through the `state` function and `StateHandle`. State changes trigger UI updates.

**Bindings**: Provide read/write access to application state via the `Binding` trait. Used to connect UI controls to state.

**Modifiers**: Chainable methods on views (via the `Modifiers` trait) that add functionality like `.padding()`, `.tap()`, `.background()`, `.size()`

**Layout**: Automatic layout system similar to SwiftUI. Uses stacks (`vstack`, `hstack`, `zstack`) for arrangement.

### File Structure

- `src/lib.rs` - Main library entry point and exports
- `src/view.rs` - Core `View` and `DynView` traits
- `src/context.rs` - Context for managing state and layout
- `src/node.rs` - The node tree: creating, matching and freeing nodes, write generations and caches
- `src/viewid.rs` - `ViewId`
- `src/views/` - Individual view implementations (button, text, shapes, etc.)
- `src/modifiers.rs` - Common view modifiers
- `src/binding.rs` - State binding system
- `src/lens.rs` - Lens system for focusing into state
- `examples/` - Example applications demonstrating usage

### State Management

State is managed through a combination of:
1. **Local State**: Using `state(initial_value, |state, cx| view)` 
2. **Bindings**: Read/write access to state via `Binding<T>`
3. **Environment**: Shared values propagated down the view tree
4. **Context**: Central storage for all state, in the node of the `state` view, addressed by `ViewId`

### Rendering Pipeline

1. **Event Processing**: Handle user input and update state
2. **Layout**: Compute view sizes and positions (cached per `state` view until a state it depends on changes)
3. **Drawing**: Render using vger (GPU vector graphics)
4. **Dirty Tracking**: Only update when state changes, skipping subtrees with no writes

### Platform Support

- **Desktop**: macOS, Windows, Linux (via winit)
- **Mobile**: iOS support available separately
- **Web**: WASM support (work in progress)

## Development Guidelines

### Creating Views
- Implement the `View` trait (which requires `DynView + Clone`)
- Use composition over direct trait implementation when possible
- Follow existing naming conventions (lowercase function names)

### State Management
- Use `state()` for component-local state
- Use `Binding<T>` for two-way data flow
- Leverage the lens system for accessing nested state

### Testing
- Examples serve as integration tests
- Unit tests exist for core functionality (bindings, lenses)
- Test by running examples and ensuring they work correctly

### Performance Considerations
- Layout is cached per `state` view and only recomputed when a state it depends on changes
- A list item whose view is a `state` view at its root is skipped while clean; any other item is rebuilt whenever its list is laid out
- Scrolling a `scroll` view isn't a state write: it moves its content's offset and asks for a redraw, so no layout is invalidated
- The `Event::Anim` pass, which runs every update, skips a list with no `anim` view in it, apart from the items written since its last layout
- A vertical or horizontal list finds the items to draw, and the items near the pointer for motion and wheel events, by binary search over their laid-out positions. It only looks at its keys after a write outside its items (or `invalidate_all`), and then compares one `Vec<u64>`. After writes only in its items (a hover, say), its layout and dirty walk visit only those items, unless one changed size. So the views around a list must not build its ids from the states inside its items
- Every walk builds the views again, so a long list should get its ids from `list_shared` with an `Arc` the app keeps, and anything derived from all its items (a filter, a sort) should be computed once per change, not on every build
- A view that reads something other than state through the `Context` (like the focus) needs `Context::invalidate_all` when it changes
- Rendering assumes 2D UI graphics are trivial for modern GPUs
- Avoid complex state dependencies that cause excessive re-renders