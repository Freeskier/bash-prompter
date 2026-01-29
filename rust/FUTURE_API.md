# Future API - Vision & Usage Examples

This document shows how the API will look once all components are implemented.

## Table of Contents

- [Simple Text Input](#simple-text-input)
- [Multi-Field Form](#multi-field-form)
- [Complex Step with Multiple Widgets](#complex-step-with-multiple-widgets)
- [Custom Widget Creation](#custom-widget-creation)
- [YAML Configuration](#yaml-configuration)
- [Complete Workflow](#complete-workflow)

---

## Simple Text Input

The simplest case - a single text input:

```rust
use rust::prelude::*;

fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    
    // Simple one-liner
    let name = runner.prompt_text("What's your name?")?.run()?;
    
    println!("Hello, {}!", name);
    
    Ok(())
}
```

## Multi-Field Form

A step with multiple input fields that can be tabbed through:

```rust
use rust::prelude::*;

fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    
    // Build a multi-field step
    let step = Step::new("Server Configuration")
        .add_field("hostname", TextInput::new()
            .with_prompt("Hostname:")
            .with_placeholder("example.com")
            .with_validation(|s| s.len() > 0))
        .add_field("port", NumberInput::new(1, 65535)
            .with_prompt("Port:")
            .with_default(8080))
        .add_field("ssl", ToggleInput::new()
            .with_prompt("Enable SSL?")
            .with_default(true));
    
    let values = runner.run(step)?;
    
    println!("Server: {}:{}", values["hostname"], values["port"]);
    println!("SSL: {}", values["ssl"]);
    
    Ok(())
}
```

**Output:**
```
❯ Server Configuration

  Hostname: [example.com___]
  Port:     [8080]
  SSL:      [✓] Yes / [ ] No

  [Tab] next • [Shift+Tab] prev • [Enter] confirm
```

## Complex Step with Multiple Widgets

Using the layout system for precise control:

```rust
use rust::prelude::*;
use rust::layout::Position;

fn build_network_config() -> Step {
    let mut widgets = WidgetRegistry::new();
    
    // Register widgets
    let date_id = widgets.register("start_date", 
        DateInput::new()
            .with_format("YYYY-MM-DD")
            .with_default(today())
    );
    
    let ip_id = widgets.register("server_ip",
        IpInput::new()
            .with_validation(|ip| !ip.is_loopback())
    );
    
    let port_id = widgets.register("port",
        NumberInput::new(1, 65535).with_default(8080)
    );
    
    let env_id = widgets.register("environment",
        SelectList::new(vec!["dev", "staging", "prod"])
            .with_default(0)
    );
    
    // Build layout with precise positioning
    let layout = Layout::new()
        .add_text("❯ Network Configuration")
        .new_line()
        .new_line_indented(2)
        .add_text("Start date: ")
        .add_widget(date_id, Position::Flow)
        .new_line_indented(2)
        .add_text("IP Address: ")
        .add_widget(ip_id, Position::Flow)
        .new_line_indented(2)
        .add_text("Port:       ")
        .add_widget(port_id, Position::Flow)
        .new_line()
        .new_line_indented(2)
        .add_text("Environment:")
        .new_line()
        .add_widget(env_id, Position::Block);
    
    Step::from_layout(layout, widgets)
}

fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    let config = runner.run(build_network_config())?;
    
    println!("Configuration complete!");
    println!("{:#?}", config);
    
    Ok(())
}
```

**Output:**
```
❯ Network Configuration

  Start date: [2024-01-15]
  IP Address: [192.168.001.100]
  Port:       [8080]
  
  Environment:
    › dev
      staging
      prod
      
  [Tab] next field • [Enter] confirm • [Ctrl+C] cancel
```

## Custom Widget Creation

How to create your own widget:

```rust
use rust::prelude::*;

/// A custom color picker widget
pub struct ColorPicker {
    colors: Vec<Color>,
    selected: usize,
    focused: bool,
}

impl Widget for ColorPicker {
    fn render(&self, focused: bool) -> WidgetRender {
        let mut lines = vec![];
        
        // Render color swatches
        let mut segments = vec![];
        for (i, color) in self.colors.iter().enumerate() {
            let marker = if i == self.selected { "●" } else { "○" };
            let style = Style::default().fg(*color);
            
            if focused && i == self.selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            
            segments.push(Segment::Styled(format!("{} ", marker), style));
        }
        
        lines.push(Line { segments });
        
        let cursor_pos = if focused {
            Some((self.selected as u16 * 2, 0))
        } else {
            None
        };
        
        WidgetRender { lines, cursor_pos }
    }
    
    fn handle_input(&mut self, event: Event) -> WidgetInputResult {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Left, .. }) => {
                if self.selected > 0 {
                    self.selected -= 1;
                    WidgetInputResult::Changed
                } else {
                    WidgetInputResult::None
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
                if self.selected < self.colors.len() - 1 {
                    self.selected += 1;
                    WidgetInputResult::Changed
                } else {
                    WidgetInputResult::None
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                WidgetInputResult::Complete
            }
            _ => WidgetInputResult::None
        }
    }
    
    fn is_interactive(&self) -> bool { true }
    fn is_valid(&self) -> bool { true }
    
    fn get_value(&self) -> Option<Value> {
        Some(Value::String(format!("{:?}", self.colors[self.selected])))
    }
    
    fn measure(&self) -> (u16, u16) {
        ((self.colors.len() * 2) as u16, 1)
    }
}

// Usage:
fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    
    let color_picker = ColorPicker::new(vec![
        Color::Red, Color::Green, Color::Blue, Color::Yellow
    ]);
    
    let step = Step::new("Choose a color")
        .add_widget("color", color_picker);
    
    let result = runner.run(step)?;
    println!("Selected: {}", result["color"]);
    
    Ok(())
}
```

## YAML Configuration

Declarative configuration (parsed into Rust structures):

```yaml
# server-setup.yml
steps:
  - input: text
    prompt: "Project name"
    variable: project_name
    validate:
      pattern: "^[a-z0-9-]+$"
      error: "Only lowercase letters, numbers, and hyphens"
  
  - input: select
    prompt: "Choose database"
    options:
      - PostgreSQL
      - MySQL
      - MongoDB
    variable: database
  
  - input: date
    prompt: "Start date"
    format: "YYYY-MM-DD"
    variable: start_date
  
  - component: object
    prompt: "Database credentials"
    fields:
      - variable: host
        display: "Host"
        input: ip
      
      - variable: port
        display: "Port"
        input: slider
        min: 1
        max: 65535
        default: 5432
      
      - variable: username
        display: "Username"
        input: text
      
      - variable: password
        display: "Password"
        input: password
    variable: db_credentials
  
  - output: info
    value: "Setup complete! Project: {{project_name}}"
```

**Usage:**

```rust
use rust::prelude::*;

fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    
    // Load from YAML
    let workflow = Workflow::from_yaml("server-setup.yml")?;
    
    // Run all steps
    let state = runner.run_workflow(workflow)?;
    
    println!("Final state:");
    println!("{:#?}", state);
    
    Ok(())
}
```

## Complete Workflow

A real-world example with conditional logic and state management:

```rust
use rust::prelude::*;

fn main() -> Result<()> {
    let mut runner = StepRunner::new()?;
    
    // Step 1: Choose deployment type
    let deploy_type = runner
        .prompt_select("Deployment type", vec!["Simple", "Advanced"])
        .run()?;
    
    // Step 2: Basic config (always)
    let basic_config = runner.run(Step::new("Basic Configuration")
        .add_field("app_name", TextInput::new()
            .with_prompt("Application name:"))
        .add_field("port", NumberInput::new(1, 65535)
            .with_prompt("Port:")
            .with_default(3000))
    )?;
    
    // Step 3: Conditional advanced config
    let advanced_config = if deploy_type == "Advanced" {
        Some(runner.run(Step::new("Advanced Configuration")
            .add_field("replicas", NumberInput::new(1, 10)
                .with_prompt("Number of replicas:")
                .with_default(3))
            .add_field("load_balancer", ToggleInput::new()
                .with_prompt("Enable load balancer?"))
            .add_field("ssl_cert", TextInput::new()
                .with_prompt("SSL certificate path:"))
        )?)
    } else {
        None
    };
    
    // Step 4: Review and confirm
    let mut review = Step::new("Review Configuration");
    review.add_static_text("Application:", &basic_config["app_name"]);
    review.add_static_text("Port:", &basic_config["port"]);
    
    if let Some(ref adv) = advanced_config {
        review.add_static_text("Replicas:", &adv["replicas"]);
        review.add_static_text("Load Balancer:", &adv["load_balancer"]);
    }
    
    let confirmed = runner.prompt_confirm("Deploy with this configuration?")
        .with_default(true)
        .run()?;
    
    if confirmed {
        // Step 5: Execute deployment
        runner.run_command("kubectl apply -f deployment.yml")
            .with_spinner(true)
            .with_message("Deploying...")
            .run()?;
        
        println!("✓ Deployment successful!");
    } else {
        println!("✗ Deployment cancelled");
    }
    
    Ok(())
}
```

## Builder Pattern API

Fluent API for quick prototyping:

```rust
use rust::prelude::*;

fn main() -> Result<()> {
    let config = StepRunner::new()?
        .prompt_text("Name")
            .with_default("John")
            .run()?
        .prompt_email("Email")
            .run()?
        .prompt_select("Role", vec!["Admin", "User", "Guest"])
            .with_default(1)
            .run()?
        .prompt_multiselect("Permissions", vec![
            "Read", "Write", "Delete", "Admin"
        ])
            .with_defaults(vec![0, 1])
            .run()?
        .prompt_date("Start date")
            .with_format("YYYY-MM-DD")
            .run()?
        .prompt_confirm("Enable notifications?")
            .with_default(true)
            .run()?;
    
    println!("Configuration: {:#?}", config);
    Ok(())
}
```

## Advanced Features

### State Management Between Steps

```rust
let mut runner = StepRunner::new()?;

// Step 1: Get database choice
let db = runner.prompt_select("Database", vec!["PostgreSQL", "MySQL"]).run()?;

// Step 2: Port depends on database choice
let default_port = match db.as_str() {
    "PostgreSQL" => 5432,
    "MySQL" => 3306,
    _ => 3306,
};

let port = runner.prompt_number("Port", 1, 65535)
    .with_default(default_port)
    .run()?;

// Access state
println!("Database: {}, Port: {}", runner.state().get("db"), port);
```

### Validation and Error Handling

```rust
let email = runner.prompt_text("Email")
    .with_validation(|s| {
        if s.contains('@') && s.contains('.') {
            Ok(())
        } else {
            Err("Invalid email format")
        }
    })
    .with_retry(3)
    .run()?;
```

### Theming

```rust
let theme = Theme::default()
    .with_primary_color(Color::Cyan)
    .with_success_color(Color::Green)
    .with_error_color(Color::Red)
    .with_prompt_prefix("❯")
    .with_selected_prefix("›");

let runner = StepRunner::new()?
    .with_theme(theme);
```

### Progress Indicators

```rust
// Spinner for long operations
runner.run_with_spinner("Installing dependencies...", || {
    // Long running operation
    install_packages()
})?;

// Progress bar
runner.run_with_progress("Downloading...", |progress| {
    for i in 0..100 {
        download_chunk(i);
        progress.update(i);
    }
})?;
```

---

## Design Goals Achieved

✅ **Inline prompts**: Steps remain visible in terminal history  
✅ **Multi-widget steps**: Tab between multiple inputs in one step  
✅ **Precise positioning**: Cell-based rendering with absolute/relative positioning  
✅ **Event-driven**: InputManager with configurable key bindings  
✅ **Extensible widgets**: Easy to create custom widget types  
✅ **Type-safe**: Rust's type system ensures correctness  
✅ **Cross-platform**: Works on Windows, macOS, Linux  
✅ **YAML support**: Declarative configuration option  
✅ **Testable**: Components can be tested independently  

---

This API design balances simplicity for common cases with flexibility for complex scenarios, all while maintaining type safety and extensibility.