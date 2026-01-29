# Quickstart Guide - Rust Terminal Prompter

## 🚀 Getting Started

This is a minimal, foundational implementation showcasing the core architecture for a modular terminal prompting library in Rust.

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- A terminal emulator

### Installation

```bash
cd rust
cargo build
```

## 🎮 Run the Demos

### Demo 1: Interactive Cursor Movement

```bash
cargo run
```

**What it demonstrates:**
- Real-time terminal size tracking
- Live cursor position updates
- Arrow key navigation
- Terminal resize handling
- Event-driven architecture

**Controls:**
- `↑ ↓ ← →` - Move cursor (yellow/green █ block)
- `Ctrl+C` - Exit

The top bar shows live terminal info: dimensions and cursor position.

### Demo 2: Advanced InputManager

```bash
cargo run --example input_manager_demo
```

**What it demonstrates:**
- Custom action definitions
- Configurable key bindings
- Event subscription system
- Multiple callbacks per action
- Action dispatch mechanism

**Controls:**
- `?` - Show help
- `Ctrl+S` - Save (triggers 2 subscribers)
- `r` - Reset counter
- `Arrow keys` - Movement actions
- `q` or `Ctrl+C` - Exit

## 📐 Architecture

### Core Components

```
┌─────────────────────────────────────────┐
│         InputManager                    │
│  • Key bindings (KeyCode → Action)     │
│  • Event dispatch (Action → Callbacks)  │
│  • Default: arrows, Ctrl+C             │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│       TerminalContext                   │
│  • Terminal size (width, height)        │
│  • Cursor position                      │
│  • Refresh on resize                    │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│          Renderer                       │
│  • Low-level terminal I/O (crossterm)   │
│  • Drawing operations                   │
│  • Cursor control                       │
│  • Raw mode management                  │
└─────────────────────────────────────────┘
```

### Key Design Decisions

1. **Event-Driven**: InputManager dispatches events to subscribers
2. **Separation of Concerns**: Each component has single responsibility
3. **Extensible**: Easy to add new actions and key bindings
4. **Cell-Based Foundation**: Precise positioning (col, row)
5. **Cross-Platform**: Uses crossterm for compatibility

## 🔧 Code Examples

### Basic Usage

```rust
use rust::{Action, InputManager, Renderer, TerminalContext};

// Initialize components
let ctx = TerminalContext::new()?;
let mut renderer = Renderer::new(ctx);
let mut input_manager = InputManager::new();

// Setup terminal
renderer.enter_raw_mode()?;
renderer.hide_cursor()?;

// Draw something
renderer.draw_at(10, 5, "Hello, World!")?;
renderer.flush()?;

// Main loop
loop {
    let event = crossterm::event::read()?;
    input_manager.handle_event(event)?;
    // ... handle actions
}

// Cleanup
renderer.show_cursor()?;
renderer.exit_raw_mode()?;
```

### Custom Key Bindings

```rust
use crossterm::event::{KeyCode, KeyModifiers};

let mut input_manager = InputManager::new();

// Add custom binding: Ctrl+S → Save
input_manager.bind(
    KeyBinding::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
    Action::Custom("save".into())
);

// Subscribe to the action
input_manager.on(Action::Custom("save".into()), |action| {
    println!("Save triggered: {:?}", action);
});
```

### Terminal Drawing

```rust
use crossterm::style::Color;

let mut renderer = Renderer::new(ctx);

// Draw colored text
renderer.draw_colored_at(
    10, 5, 
    "Hello!", 
    Color::Cyan, 
    Some(Color::DarkGrey)
)?;

// Draw a cell (single character with colors)
renderer.draw_cell(20, 5, '█', Color::Yellow, Color::Green)?;

// Get terminal info
let width = renderer.context().width();
let height = renderer.context().height();
```

## 📁 Project Structure

```
src/
├── lib.rs                  # Public API exports
├── main.rs                 # Interactive cursor demo
├── terminal_context.rs     # Terminal state tracking
├── input_manager.rs        # Event handling & key bindings
├── renderer.rs             # Terminal drawing operations
└── TARGET_IDEA.md          # Complete architecture plan

examples/
└── input_manager_demo.rs   # Advanced event handling demo
```

## 🎯 What's Next?

This is Phase 1 of the architecture. See `TARGET_IDEA.md` for the full plan:

### Upcoming Phases

- **Phase 2**: RenderBuffer (2D cell grid for complex layouts)
- **Phase 3**: Widget System (TextInput, Select, DatePicker, etc.)
- **Phase 4**: Layout System (positioning, flow control)
- **Phase 5**: Scene/Step (multi-widget interactive prompts)
- **Phase 6**: YAML Parser (declarative configuration)

### Current Status

✅ TerminalContext - Query terminal state  
✅ InputManager - Event handling with key bindings  
✅ Renderer - Low-level drawing operations  
⬜ RenderBuffer - 2D cell grid  
⬜ Widget trait - UI components  
⬜ Layout system - Positioning  
⬜ Scene - Multi-widget prompts  
⬜ YAML parser - Declarative config  

## 🧪 Testing

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Check code
cargo clippy
cargo fmt --check
```

## 📚 Learn More

- **TARGET_IDEA.md** - Complete architecture documentation
- **examples/** - Working code examples
- **Bash version** - See `../IMPLEMENTATION_GUIDE.md` for original implementation

## 🤝 Contributing

This is a foundational implementation. Key areas for contribution:

1. Implement RenderBuffer (2D cell grid)
2. Create basic Widget trait
3. Build simple widgets (TextInput, Select)
4. Add Layout positioning system
5. Implement Scene execution

See TARGET_IDEA.md for detailed specifications.

## 📝 License

MIT