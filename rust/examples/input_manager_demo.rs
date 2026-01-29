//! Advanced InputManager Demo
//!
//! This example demonstrates how to:
//! - Define custom actions
//! - Create custom key bindings
//! - Subscribe to actions with callbacks
//! - Handle multiple subscribers for the same action

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use rust::{Action, InputManager, KeyBinding, Renderer, TerminalContext};
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== InputManager Advanced Demo ===\n");

    // Create input manager
    let mut input_manager = InputManager::new();

    // Counter to demonstrate state changes
    let mut action_count = 0;

    // Define custom actions and bindings
    println!("Setting up custom key bindings...\n");

    // Custom action: Quit with 'q'
    input_manager.bind(
        KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE),
        Action::Custom("quit".into()),
    );

    // Custom action: Save with Ctrl+S
    input_manager.bind(
        KeyBinding::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        Action::Custom("save".into()),
    );

    // Custom action: Help with '?'
    input_manager.bind(
        KeyBinding::new(KeyCode::Char('?'), KeyModifiers::NONE),
        Action::Custom("help".into()),
    );

    // Custom action: Reset with 'r'
    input_manager.bind(
        KeyBinding::new(KeyCode::Char('r'), KeyModifiers::NONE),
        Action::Custom("reset".into()),
    );

    // Subscribe to actions with callbacks
    println!("Subscribing to actions...\n");

    // Multiple subscribers can listen to the same action
    input_manager.on(Action::Custom("save".into()), |action| {
        println!("📁 [Subscriber 1] Save action triggered: {:?}", action);
    });

    input_manager.on(Action::Custom("save".into()), |action| {
        println!("💾 [Subscriber 2] Also handling save: {:?}", action);
    });

    input_manager.on(Action::Custom("help".into()), |_| {
        println!("\n=== HELP ===");
        println!("  q          - Quit");
        println!("  Ctrl+S     - Save");
        println!("  Ctrl+C     - Exit");
        println!("  r          - Reset counter");
        println!("  ?          - Show this help");
        println!("  Arrow keys - Movement actions");
        println!("============\n");
    });

    input_manager.on(Action::Custom("reset".into()), |_| {
        println!("🔄 Counter reset!");
    });

    input_manager.on(Action::MoveUp, |_| {
        println!("⬆️  Move up");
    });

    input_manager.on(Action::MoveDown, |_| {
        println!("⬇️  Move down");
    });

    input_manager.on(Action::MoveLeft, |_| {
        println!("⬅️  Move left");
    });

    input_manager.on(Action::MoveRight, |_| {
        println!("➡️  Move right");
    });

    // Setup terminal
    let ctx = TerminalContext::new()?;
    let mut renderer = Renderer::new(ctx);
    renderer.enter_raw_mode()?;

    println!("Press '?' for help, 'q' or Ctrl+C to exit\n");
    println!("Try pressing arrow keys, Ctrl+S, 'r', etc.\n");

    let mut should_exit = false;

    // Main event loop
    loop {
        if should_exit {
            break;
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            let terminal_event = event::read()?;

            match terminal_event {
                Event::Key(key_event) => {
                    let binding = KeyBinding::from_key_event(key_event);

                    // Check if it's an exit action
                    if let Some(Action::Exit) = input_manager.bindings.get(&binding) {
                        println!("\n👋 Exiting via Ctrl+C...");
                        should_exit = true;
                        continue;
                    }

                    // Check if it's a quit action
                    if let Some(Action::Custom(name)) = input_manager.bindings.get(&binding) {
                        if name == "quit" {
                            println!("\n👋 Exiting via 'q'...");
                            should_exit = true;
                            continue;
                        }

                        if name == "reset" {
                            action_count = 0;
                        }
                    }

                    // Handle the event (will dispatch to subscribers)
                    let handled = input_manager.handle_event(Event::Key(key_event))?;

                    if handled {
                        action_count += 1;
                        println!("  [Total actions handled: {}]\n", action_count);
                    } else {
                        println!("  ⚠️  Unbound key: {:?}\n", key_event);
                    }
                }
                Event::Resize(w, h) => {
                    println!("📐 Terminal resized to: {}x{}\n", w, h);
                }
                _ => {}
            }
        }
    }

    // Cleanup
    renderer.exit_raw_mode()?;

    println!("\nDemo completed! Total actions handled: {}", action_count);

    Ok(())
}
