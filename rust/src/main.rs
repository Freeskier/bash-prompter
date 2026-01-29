//! Interactive Demo with Node System and Layout
//!
//! This demo showcases the clean architecture:
//! - Node trait: Base abstraction for renderable elements
//! - TextNode: Static text with automatic width/height and newline support
//! - Layout: Manages nodes and calculates their positions automatically
//! - Renderer: Draws nodes at their calculated positions
//! - TerminalContext: Tracks terminal size and state
//! - InputManager: Configurable key bindings
//!
//! Controls:
//! - Arrow keys: Move the cursor
//! - Ctrl+C: Exit the application

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    style::Color,
};
use rust::{Action, InputManager, Layout, Renderer, TerminalContext, TextNode};
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize components
    let ctx = TerminalContext::new()?;
    let mut renderer = Renderer::new(ctx);
    let mut input_manager = InputManager::new();

    // Track cursor position
    let mut cursor_x = 0u16;
    let mut cursor_y = 0u16;
    let mut previous_render_height = 0u16; // Track previous render height for clearing

    // Setup terminal
    renderer.enter_raw_mode()?;
    renderer.hide_cursor()?;

    // Get actual cursor position after raw mode
    let (_, cursor_start) = crossterm::cursor::position()?;
    let mut current_row: i32 = cursor_start as i32;

    // Create layout starting at current position
    let mut layout = Layout::new(cursor_start);

    // Initial render
    previous_render_height = render_ui(
        &mut layout,
        &mut renderer,
        cursor_x,
        cursor_y,
        current_row.max(0) as u16,
        previous_render_height,
    )?;

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
                Event::Resize(_, _) => {
                    renderer.refresh_context()?;
                    previous_render_height = render_ui(
                        &mut layout,
                        &mut renderer,
                        cursor_x,
                        cursor_y,
                        current_row.max(0) as u16,
                        previous_render_height,
                    )?;
                }
                Event::Key(key_event) => {
                    let binding = rust::input_manager::KeyBinding::from_key_event(key_event);

                    // Check for exit
                    if let Some(Action::Exit) = input_manager.bindings.get(&binding) {
                        should_exit = true;
                        continue;
                    }

                    // Handle movement
                    if let Some(action) = input_manager.bindings.get(&binding) {
                        let term_width = renderer.context().width();
                        let term_height = renderer.context().height();

                        match action {
                            Action::MoveUp => {
                                if cursor_y > 0 {
                                    cursor_y -= 1;
                                }
                            }
                            Action::MoveDown => {
                                cursor_y += 1;

                                // Check if we need to expand downwards
                                let absolute_row =
                                    (current_row + 1 + cursor_y as i32).max(0) as u16;
                                if absolute_row >= term_height {
                                    // Would cause scroll - add newline
                                    renderer.draw("\n")?;
                                    renderer.flush()?;

                                    // After scroll, start_row moves up by 1
                                    // Can go negative (means content is above viewport)
                                    current_row -= 1;
                                    if current_row >= 0 {
                                        layout.set_start_row(current_row as u16);
                                    }
                                }
                            }
                            Action::MoveLeft => {
                                if cursor_x > 0 {
                                    cursor_x -= 1;
                                }
                            }
                            Action::MoveRight => {
                                if cursor_x < term_width - 1 {
                                    cursor_x += 1;
                                }
                            }
                            _ => {}
                        }

                        previous_render_height = render_ui(
                            &mut layout,
                            &mut renderer,
                            cursor_x,
                            cursor_y,
                            current_row.max(0) as u16,
                            previous_render_height,
                        )?;
                    }

                    // Handle event through input manager
                    input_manager.handle_event(Event::Key(key_event))?;
                }
                _ => {}
            }
        }
    }

    // Cleanup - move cursor below UI
    let final_row = layout
        .nodes()
        .last()
        .map_or(current_row.max(0) as u16, |node| {
            let (_, y) = node.position();
            y + node.height()
        });
    renderer.move_to(0, final_row.saturating_add(1))?;
    renderer.show_cursor()?;
    renderer.exit_raw_mode()?;
    renderer.draw("\n")?;
    renderer.flush()?;

    Ok(())
}

/// Render the entire UI using Layout
/// Returns the height of the rendered content
fn render_ui(
    layout: &mut Layout,
    renderer: &mut Renderer,
    cursor_x: u16,
    cursor_y: u16,
    start_row: u16,
    previous_height: u16,
) -> Result<u16> {
    let width = renderer.context().width();
    let height = renderer.context().height();

    // First, clear all lines from previous render (only visible ones)
    let clear_start = start_row;
    let clear_end = (start_row + previous_height).min(height);
    for y in clear_start..clear_end {
        renderer.move_to(0, y)?;
        renderer.clear_line()?;
    }

    // Clear layout and rebuild
    layout.clear();

    // Update layout start_row in case it changed due to scroll
    layout.set_start_row(start_row);

    // Create info node (only if start_row is visible)
    let info_text = format!(
        "Terminal: {}x{} | Cursor: ({}, {}) | Start: {} | Press Ctrl+C to exit | Use arrow keys to move",
        width, height, cursor_x, cursor_y, start_row
    );
    let info_node = TextNode::new(info_text).with_colors(Color::Cyan, Color::DarkGrey);
    layout.add(Box::new(info_node));

    // Add cursor node with vertical offset
    // Build multiline text with cursor at the right position
    let mut cursor_lines = Vec::new();
    for i in 0..=cursor_y {
        if i == cursor_y {
            let cursor_line = format!("{:width$}█", "", width = cursor_x as usize);
            cursor_lines.push(cursor_line);
        } else {
            cursor_lines.push(String::new());
        }
    }

    let cursor_text = cursor_lines.join("\n");
    let cursor_node = TextNode::new(cursor_text)
        .with_colors(Color::Yellow, Color::Green)
        .with_new_line(true);

    layout.add(Box::new(cursor_node));

    // Render the layout
    layout.render(renderer)?;

    // Calculate and return the height of what we just rendered
    let rendered_height = layout.total_height();
    Ok(rendered_height)
}
