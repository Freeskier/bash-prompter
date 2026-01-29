# Target Architecture - Rust Terminal Prompter

## Overview

This document outlines the architecture for a modular, extensible terminal prompting library in Rust. The goal is to create inline, interactive prompts (not full-screen TUIs) where:
- Each prompt/step is interactive and can contain multiple input widgets
- Previous steps remain visible in terminal history
- Only the current step is "live" and can be redrawn
- Easy to add new input types and components
- Precise control over widget positioning

## Core Design Principles

1. **Inline, not full-screen**: Prompts render inline in the terminal. Completed steps become part of terminal history.
2. **Cell-based rendering**: Use a 2D cell buffer for precise positioning, not just string concatenation.
3. **Component-based**: Each input type (text, select, date picker, IP address) is an independent, reusable widget.
4. **Separation of concerns**: Clear boundaries between terminal I/O, rendering, layout, event handling, and business logic.
5. **Extensibility**: Adding new widget types should be straightforward.

## Architecture Components

### 1. TerminalContext
**Responsibility**: Query terminal capabilities and state (read-only)

```rust
pub struct TerminalContext {
    width: u16,
    height: u16,
    cursor_pos: (u16, u16),
    supports_colors: bool,
}
```

- Tracks terminal dimensions
- Can be refreshed when terminal resizes
- Provides queries like `can_fit(lines)`, `width()`, `height()`
- Does NOT perform any I/O operations itself

### 2. RenderBuffer
**Responsibility**: 2D grid of cells representing what should be displayed

```rust
pub struct Cell {
    ch: char,
    style: Style,  // color, bold, italic, etc.
}

pub struct RenderBuffer {
    cells: Vec<Vec<Cell>>,  // [row][col]
    width: usize,
    height: usize,
    cursor_pos: Option<(u16, u16)>,
}
```

Key operations:
- `set_cell(col, row, cell)` - Set individual cell
- `write_str(col, row, text, style)` - Write string at position
- `write_segments(col, row, segments)` - Write styled segments
- `render_widget(col, row, widget, focused)` - Render widget at position
- `diff(previous)` - Calculate what changed (optimization)

**Why cell-based?**
- Precise positioning: can place widget at column 30 without 30 spaces
- Efficient updates: only redraw changed cells
- Style management: each cell knows its style
- Foundation for more complex layouts

### 3. Renderer
**Responsibility**: Low-level terminal I/O operations using crossterm

```rust
pub struct Renderer {
    stdout: Stdout,
    ctx: TerminalContext,
}
```

Operations:
- `clear_region(start_line, num_lines)` - Clear area
- `draw_buffer(buffer, start_y)` - Render buffer to terminal
- `draw_cell_updates(updates)` - Draw only changed cells (optimization)
- `set_cursor(x, y)` - Position cursor
- `hide_cursor()` / `show_cursor()` - Cursor visibility

Uses crossterm for actual terminal operations. This is the ONLY component that talks to crossterm directly.

### 4. Widget Trait
**Responsibility**: Individual interactive/static components

```rust
pub trait Widget {
    fn render(&self, focused: bool) -> WidgetRender;
    fn handle_input(&mut self, event: Event) -> WidgetInputResult;
    fn is_interactive(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn get_value(&self) -> Option<Value>;
    fn measure(&self) -> (u16, u16);  // (width, height)
}

pub struct WidgetRender {
    lines: Vec<Line>,
    cursor_pos: Option<(u16, u16)>,  // relative to widget
}

pub struct Line {
    segments: Vec<Segment>,
}

pub enum Segment {
    Text(String),
    Styled(String, Style),
    Spacer(u16),
}

pub enum WidgetInputResult {
    Changed,    // Widget state changed, needs redraw
    Complete,   // Widget is done, move to next
    None,       // Event not handled
}
```

Example widgets:
- `TextInput`: Simple text field with validation
- `SelectList`: Arrow-navigable list (single choice)
- `MultiSelect`: Checkbox-style list (multiple choices)
- `DateInput`: Date picker with field navigation (year/month/day)
- `IpInput`: IP address with octet navigation
- `SliderInput`: Numeric slider with arrow keys
- `ToggleInput`: Boolean yes/no
- `PasswordInput`: Masked text input

Each widget:
- Knows how to render itself (focused vs unfocused)
- Handles its own input events
- Reports its size
- Validates its own data

### 5. Layout
**Responsibility**: Logical positioning of elements

```rust
pub struct Layout {
    elements: Vec<PositionedElement>,
}

pub struct PositionedElement {
    position: Position,
    element: Element,
}

pub enum Position {
    Absolute { col: usize, row: usize },      // Fixed position
    Relative { col_offset: isize, row_offset: isize },  // Relative to previous
    Flow,                                      // Normal flow (like HTML)
    NextLine { indent: usize },               // New line with indent
}

pub enum Element {
    Text { content: String, style: Style },
    Widget { id: WidgetId },
    Box { width, height, border, content },   // Container
}
```

Example usage:
```rust
let layout = Layout::new()
    .add_text("❯ Configure network:")
    .new_line()
    .add_text("  Date: ")
    .add_widget(date_widget_id)  // inline widget
    .new_line()
    .add_text("  IP: ")
    .add_widget_at(8, 2, ip_widget_id)  // absolute positioning
    .new_line_indented(2)
    .add_widget(select_widget_id);  // block widget
```

Layout is **terminal-agnostic** - it doesn't know about actual rendering, just logical structure.

### 6. WidgetRegistry
**Responsibility**: Manage collection of widgets for a scene

```rust
pub struct WidgetRegistry {
    widgets: HashMap<WidgetId, Box<dyn Widget>>,
    names: HashMap<WidgetId, String>,
}
```

Operations:
- `register(name, widget) -> WidgetId` - Add widget, get ID
- `get(id)` / `get_mut(id)` - Access widgets
- `interactive_ids()` - Get IDs of interactive widgets (for tab order)
- `all_valid()` - Check if all widgets are valid
- `collect_values()` - Get HashMap of all widget values

Separates widget lifecycle from layout structure.

### 7. InputManager
**Responsibility**: Central event handling and routing

```rust
pub struct InputManager {
    focused_widget: WidgetId,
    widget_order: Vec<WidgetId>,  // Tab order
}

pub enum InputAction {
    FocusNext,                    // Tab pressed
    FocusPrev,                    // Shift+Tab pressed
    Exit,                         // Ctrl+C pressed
    Widget(WidgetId, Event),      // Route to widget
    Complete,                     // Force complete (Ctrl+Enter)
    Resize,                       // Terminal resized
    None,
}
```

Handles global shortcuts:
- `Ctrl+C` → Exit
- `Tab` → Next widget
- `Shift+Tab` → Previous widget
- `Ctrl+Enter` → Force complete step
- Terminal resize events

Routes other events to focused widget.

### 8. Scene (Step)
**Responsibility**: Integrates everything - main execution unit

```rust
pub struct Scene {
    layout: Layout,
    widgets: WidgetRegistry,
    input_manager: InputManager,
    region_start: u16,
    needs_redraw: bool,
    previous_buffer: Option<RenderBuffer>,
}
```

Main loop:
```rust
pub fn run(&mut self, renderer: &mut Renderer) -> Result<SceneResult> {
    renderer.hide_cursor()?;
    
    loop {
        if self.needs_redraw {
            self.redraw(renderer)?;
        }
        
        let event = crossterm::event::read()?;
        let action = self.input_manager.handle_event(event)?;
        
        match action {
            InputAction::Exit => return Ok(SceneResult::Cancelled),
            InputAction::FocusNext => { ... }
            InputAction::Widget(id, event) => {
                let result = self.widgets.get_mut(id).handle_input(event);
                if result == Changed { self.needs_redraw = true; }
            }
            InputAction::Complete => {
                if self.all_valid() {
                    return Ok(SceneResult::Complete(self.collect_values()));
                }
            }
            ...
        }
    }
}

fn redraw(&mut self, renderer: &mut Renderer) -> Result<()> {
    let buffer = self.render_to_buffer();
    
    // Option 1: Full redraw
    renderer.draw_buffer(&buffer, self.region_start)?;
    
    // Option 2: Differential update (optimization)
    if let Some(prev) = &self.previous_buffer {
        let updates = buffer.diff(prev);
        renderer.draw_cell_updates(&updates)?;
    }
    
    self.previous_buffer = Some(buffer);
}
```

Scene represents one "step" in your workflow. When complete:
- Leave output in terminal (becomes history)
- Move cursor down
- Return collected values

### 9. StepRunner
**Responsibility**: Execute sequence of steps

```rust
pub struct StepRunner {
    ctx: TerminalContext,
    renderer: Renderer,
    state: State,  // Variables between steps
    history: Vec<CompletedStep>,
}

pub fn run_workflow(&mut self, steps: Vec<StepBuilder>) -> Result<State> {
    for step_builder in steps {
        let mut scene = step_builder.build(&self.state)?;
        
        match scene.run(&mut self.renderer)? {
            SceneResult::Complete(values) => {
                self.state.merge(values);
                self.history.push(...);
            }
            SceneResult::Cancelled => {
                return Err(Error::Cancelled);
            }
        }
    }
    
    Ok(self.state)
}
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         StepRunner                          │
│  - Executes sequence of scenes                              │
│  - Manages global state                                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                           Scene                             │
│  - Main event loop                                          │
│  - Coordinates all components                               │
└──┬────────┬─────────┬─────────┬─────────┬──────────────────┘
   │        │         │         │         │
   ▼        ▼         ▼         ▼         ▼
┌──────┐ ┌────────┐ ┌──────┐ ┌────────┐ ┌──────────┐
│Layout│ │Widgets │ │Input │ │Renderer│ │Terminal  │
│      │ │Registry│ │Manager│ │        │ │Context   │
└──┬───┘ └───┬────┘ └───┬───┘ └───┬────┘ └─────┬────┘
   │         │           │         │            │
   │         │           │         │            │
   ▼         ▼           ▼         ▼            ▼
┌─────────────────────────────────────────────────────┐
│                  RenderBuffer                       │
│  - 2D cell grid                                     │
│  - Cursor position                                  │
└─────────────────────────────────────────────────────┘
                       │
                       ▼
               ┌───────────────┐
               │   Terminal    │
               │   (crossterm) │
               └───────────────┘
```

## Rendering Pipeline

1. **Scene determines redraw needed**
   - User input changed widget state
   - Focus changed
   - Terminal resized

2. **Create RenderBuffer**
   - Allocate 2D cell grid (size from TerminalContext)

3. **Layout renders to buffer**
   - Iterate through positioned elements
   - Resolve positions (absolute, relative, flow)
   - For text: write directly to buffer
   - For widgets: call `widget.render()`, write result to buffer

4. **Buffer → Terminal**
   - Option A: Full redraw (clear region + draw all)
   - Option B: Differential (only changed cells)

5. **Position cursor**
   - Focused widget reports cursor position
   - Convert to absolute coordinates
   - Update terminal cursor

## Example: Multi-Field Input Step

```rust
fn build_server_config() -> Scene {
    let mut widgets = WidgetRegistry::new();
    
    let date_id = widgets.register("start_date", Box::new(
        DateInput::new()
            .with_format("YYYY-MM-DD")
            .with_default(today())
    ));
    
    let ip_id = widgets.register("server_ip", Box::new(
        IpInput::new()
            .with_validation(|ip| !ip.is_loopback())
    ));
    
    let port_id = widgets.register("port", Box::new(
        NumberInput::new(1, 65535)
            .with_default(8080)
    ));
    
    let env_id = widgets.register("environment", Box::new(
        SelectList::new(vec!["dev", "staging", "prod"])
    ));
    
    let layout = Layout::new()
        .add_text("❯ Server Configuration")
        .new_line()
        .new_line_indented(2)
        .add_text("Start date: ")
        .add_widget(date_id)
        .new_line_indented(2)
        .add_text("IP Address: ")
        .add_widget(ip_id)
        .new_line_indented(2)
        .add_text("Port:       ")
        .add_widget(port_id)
        .new_line()
        .new_line_indented(2)
        .add_text("Environment:")
        .new_line()
        .add_widget(env_id);  // Block widget, takes full lines
    
    Scene::new(layout, widgets)
}
```

When running:
```
❯ Server Configuration

  Start date: [2024-01-15]     ← Tab to focus
  IP Address: [192.168.001.100] ← Tab to focus
  Port:       [8080]            ← Tab to focus
  
  Environment:
    › dev                       ← Tab to focus
      staging
      prod
      
  [Tab] next field • [Shift+Tab] prev • [Enter] confirm • [Ctrl+C] cancel
```

Each field is independently focusable with Tab. Arrow keys, typing, etc. work within focused widget. When all valid and Enter pressed, scene completes and values returned.

## Widget Examples

### DateInput (inline widget)
- Renders as `[YYYY-MM-DD]`
- Tab/Arrow keys switch between year/month/day fields
- Up/Down increment/decrement current field
- Validates date correctness
- Can customize format

### IpInput (inline widget)
- Renders as `[192.168.001.100]`
- Arrow keys move between octets
- Type digits to edit current octet
- Validates each octet (0-255)
- Auto-advances to next octet

### SelectList (block widget)
- Renders multiple lines with arrow indicator
- Up/Down arrows navigate
- Enter selects
- Can support filtering/search
- Returns selected value

### TextInput (inline widget)
- Simple text field
- Regex validation
- Placeholder text when empty
- Can limit length

## Future Enhancements

### Conditional Steps
```rust
let choice = run_step(select_step)?;

if choice == "advanced" {
    run_step(advanced_config_step)?;
} else {
    run_step(simple_config_step)?;
}
```

### YAML/Config Parser
Later, can parse YAML files to generate scenes:
```yaml
steps:
  - input: date
    prompt: "Start date"
    variable: start_date
    
  - input: ip
    prompt: "Server IP"
    variable: server_ip
    
  - component: select
    prompt: "Environment"
    options: [dev, staging, prod]
    variable: env
```

Parser creates appropriate widgets and layouts from declarative config.

### Nested Components
```rust
ObjectComponent {
    fields: vec![
        Field::new("hostname", TextInput::new()),
        Field::new("port", NumberInput::new(1, 65535)),
        Field::new("auth", ObjectComponent {
            fields: vec![
                Field::new("username", TextInput::new()),
                Field::new("password", PasswordInput::new()),
            ]
        }),
    ]
}
```

## File Structure

```
src/
  lib.rs                      - Public API
  
  core/
    terminal_context.rs       - Terminal state queries
    render_buffer.rs          - Cell-based 2D buffer
    renderer.rs               - Terminal I/O (crossterm wrapper)
    
  layout/
    mod.rs                    - Layout, Position, Element
    cursor.rs                 - Layout cursor for flow positioning
    
  widget/
    mod.rs                    - Widget trait
    text.rs                   - TextInput
    select.rs                 - SelectList
    multiselect.rs            - MultiSelect
    date.rs                   - DateInput
    ip.rs                     - IpInput
    number.rs                 - NumberInput/Slider
    toggle.rs                 - ToggleInput
    password.rs               - PasswordInput
    object.rs                 - ObjectComponent (form)
    
  scene/
    mod.rs                    - Scene
    registry.rs               - WidgetRegistry
    result.rs                 - SceneResult
    
  input/
    mod.rs                    - InputManager
    action.rs                 - InputAction
    
  runner/
    mod.rs                    - StepRunner
    state.rs                  - State management
    
  style/
    mod.rs                    - Style, Color definitions
    theme.rs                  - Theming support
    
  validation/
    mod.rs                    - Validator trait
    patterns.rs               - Common validators (email, url, etc.)
```

## Key Decisions

### Why cell-based rendering?
- **Precise positioning**: Place widgets at exact column/row
- **Efficient updates**: Only redraw changed cells
- **Style management**: Each cell carries its own style
- **Foundation for complex layouts**: Tables, boxes, etc.

### Why inline vs full-screen TUI?
- **History preservation**: Users can scroll back to see previous answers
- **Familiar UX**: Like command-line tools (npm init, etc.)
- **Simpler state**: Only current step is "live"
- **Better for workflows**: Linear progression through steps

### Why separate Layout from Renderer?
- **Testability**: Layout logic can be tested without terminal
- **Flexibility**: Same layout could render to different targets
- **Clarity**: Logical structure vs physical rendering are separate concerns

### Why Widget trait vs enum?
- **Extensibility**: Easy to add new widget types (just impl trait)
- **Encapsulation**: Each widget manages its own state and logic
- **Reusability**: Widgets are self-contained and composable

## Testing Strategy

### Unit Tests
- Widget rendering (assert on WidgetRender output)
- Widget input handling (send events, check state)
- Layout positioning logic
- Buffer operations (set_cell, write_str, diff)

### Integration Tests
- Full scene execution (mock terminal)
- Tab navigation between widgets
- Validation and completion logic

### Visual Tests
- Example programs for each widget type
- Manual testing in different terminals
- Color/style verification

## Dependencies

- **crossterm**: Terminal manipulation (cross-platform)
- **unicode-width**: Correct width calculation for Unicode
- **chrono**: Date/time handling (for DateInput)
- **regex**: Pattern validation
- **serde** (optional): For YAML parser later

## Migration from Bash Version

The bash implementation has:
- YAML parser → Will be implemented later
- Input types → Map to widgets
- Components → Map to complex widgets
- State management → StepRunner.state
- Validation → Widget.is_valid()

This Rust version provides:
- Better type safety
- More extensible architecture
- Cross-platform support
- Better performance
- Easier testing

## Next Steps

1. **Phase 1: Core infrastructure**
   - TerminalContext
   - RenderBuffer
   - Renderer
   - Basic Widget trait

2. **Phase 2: Simple widgets**
   - TextInput
   - SelectList
   - NumberInput

3. **Phase 3: Scene & InputManager**
   - Scene execution loop
   - Focus management
   - Event routing

4. **Phase 4: Complex widgets**
   - DateInput
   - IpInput
   - MultiSelect
   - ObjectComponent

5. **Phase 5: Layout system**
   - Positioning logic
   - Flow layout
   - Box containers

6. **Phase 6: YAML parser**
   - Parse declarative config
   - Generate scenes from YAML
   - Match bash version functionality

---

This architecture provides a solid foundation for building complex, interactive terminal prompts while maintaining clean separation of concerns and extensibility.