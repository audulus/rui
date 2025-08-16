# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rui is an experimental declarative UI library for Rust, inspired by SwiftUI. It focuses on GPU-rendered UIs that update reactively when state changes, without a retained view tree or DOM diffing. The entire UI is re-rendered when state changes, under the assumption that this is fast enough for good performance.

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

**Context**: Stores all mutable UI state, keyed by `ViewId`s. The `Context` manages:
- Layout information for all views
- User state created by `state` functions
- Touch/mouse interaction state
- Environment values
- Dirty tracking for reactive updates

**ViewId System**: Each view gets a unique identifier (u64) formed by hashing the traversal path down the view tree. This enables efficient state storage and retrieval.

**Reactive Updates**: The entire UI re-renders when state changes. Multiple state changes in a single event cycle are coalesced for efficiency.

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
4. **Context**: Central storage for all state, indexed by ViewId

### Rendering Pipeline

1. **Event Processing**: Handle user input and update state
2. **Layout**: Compute view sizes and positions (cached until state changes)  
3. **Drawing**: Render using vger (GPU vector graphics)
4. **Dirty Tracking**: Only re-render when state actually changes

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
- Layout is cached and only recomputed when state changes
- Rendering assumes 2D UI graphics are trivial for modern GPUs
- Avoid complex state dependencies that cause excessive re-renders