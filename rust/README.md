# Rust Terminal Prompter - Demo

A modular, extensible terminal prompting library in Rust with precise cursor control and event-driven architecture.

## Demo: Interactive Cursor Movement

This demo showcases the foundational components:

- **TerminalContext**: Tracks terminal size and cursor position with live updates
- **InputManager**: Configurable key bindings with event dispatch system
- **Renderer**: Low-level terminal drawing operations using crossterm

### Features Demonstrated

1. **Live Terminal Info**: Top bar shows terminal dimensions (width × height) and cursor position, updated in real-time
2. **Cursor Movement**: Use arrow keys to move a filled block character (█) around the terminal
3. **Terminal Resize**: Info updates automatically when you resize the terminal window
4. **Event System**: Key bindings trigger actions that can be subscribed to by other components

### Running the Demo

```bash
cd rust
cargo run
```

### Controls

- **Arrow Keys** (↑ ↓ ← →): Move the cursor
- **Ctrl+C**: Exit the application

### What You'll See

```
Terminal: 80x24 | Cursor: (10, 5) | Press Ctrl+C to exit | Use arrow keys to move
                    █
                    
                    
                    
Navigation: ← ↑ → ↓  |  Exit: Ctrl+C
```

The yellow/green block (█) represents your cursor position and moves as you press arrow keys.

## Examples

### 1. Main Demo - Interactive Cursor Movement

```bash
cargo run
```

This is the main demo showcasing real-time terminal interaction with cursor movement.

### 2. InputManager Demo - Advanced Event Handling

```bash
cargo run --example input_manager_demo
```

This example demonstrates:
- Custom action definitions
- Multiple key bindings
- Event subscription with callbacks
- Multiple subscribers for the same action

Try pressing:
- `?` - Show help
- `Ctrl+S` - Trigger save action (with multiple subscribers)
- `r` - Reset counter
- Arrow keys - Movement actions
- `q` or `Ctrl+C` - Exit

The yellow/green block (█) represents your cursor position and moves as you press arrow keys.

## Architecture Overview

### Components Implemented

#### 1. TerminalContext (`terminal_context.rs`)
- Queries and tracks terminal state
- Dimensions (width, height)
- Cursor position
- Capabilities (color support)
- Refresh on terminal resize

#### 2. InputManager (`input_manager.rs`)
- Configurable key bindings (KeyBinding → Action)
- Event listener system (Action → Callbacks)
- Default bindings for arrow keys and Ctrl+C
- Easy to add custom key combinations and actions

```rust
// Example: Define custom action
let mut input_manager = InputManager::new();

// Add custom binding
input_manager.bind(
    KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE),
    Action::Custom("quit".into())
);

// Subscribe to action
input_manager.on(Action::Custom("quit".into()), |action| {
    println!("Quit action triggered: {:?}", action);
});
```

#### 3. Renderer (`renderer.rs`)
- Low-level terminal I/O using crossterm
- Drawing operations (text, colored text, cells)
- Cursor positioning and visibility
- Screen clearing
- Raw mode management

### Key Design Decisions

1. **Separation of Concerns**: Each component has a single, clear responsibility
2. **Event-Driven**: InputManager dispatches events to subscribers
3. **Cell-Based**: Foundation for precise positioning (x, y coordinates)
4. **Cross-Platform**: Uses crossterm for terminal compatibility

## Next Steps

This demo establishes the foundation. Future implementations will include:

- **RenderBuffer**: 2D cell grid for complex layouts
- **Widget System**: Reusable UI components (TextInput, Select, DatePicker, etc.)
- **Layout System**: Positioning and flow control
- **Scene/Step**: Complete interactive prompts with multiple widgets
- **YAML Parser**: Declarative configuration for prompts

See `TARGET_IDEA.md` for the complete architecture plan.

## Development

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Run Examples
```bash
# Basic cursor movement demo
cargo run

# Advanced InputManager demo
cargo run --example input_manager_demo
```

### Test
```bash
cargo test
```

## Dependencies

- `crossterm` (0.27): Cross-platform terminal manipulation
- `anyhow` (1): Error handling

## License

MIT