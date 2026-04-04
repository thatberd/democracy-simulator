use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::PathBuf;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::engine::{Simulation, State};
use crate::config::SimConfig;
use super::renderer::UIRenderer;

pub struct App {
    simulation: Simulation,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    should_quit: bool,
    event_scroll_offset: usize,
    auto_save_path: Option<PathBuf>,
}

impl App {
    pub fn new(seed: u64) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            simulation: Simulation::new(seed),
            terminal,
            should_quit: false,
            event_scroll_offset: 0,
            auto_save_path: None,
        })
    }

    pub fn new_with_config(seed: u64, config: SimConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            simulation: Simulation::new_with_config(seed, config),
            terminal,
            should_quit: false,
            event_scroll_offset: 0,
            auto_save_path: None,
        })
    }

    /// Create a new App with an existing state (for loading saved simulations)
    pub fn new_with_state(state: State) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create simulation from state
        let simulation = Simulation::from_state(state);

        Ok(Self {
            simulation,
            terminal,
            should_quit: false,
            event_scroll_offset: 0,
            auto_save_path: None,
        })
    }

    /// Set auto-save path for periodic saving
    pub fn set_auto_save_path(&mut self, path: String) {
        self.auto_save_path = Some(PathBuf::from(path));
    }

    /// Save current simulation state to file
    pub fn save_state(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = self.simulation.state().serialize_state()?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_tick = std::time::Instant::now();
        let tick_duration = std::time::Duration::from_millis(100); // 10 FPS

        loop {
            // Handle input with single event read
            if event::poll(std::time::Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(key) => {
                        // Only process key press events, not release events (fixes Windows double input)
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key);
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse);
                    }
                    _ => {}
                }
            }

            // Update simulation at fixed rate
            if last_tick.elapsed() >= tick_duration {
                self.simulation.tick();
                last_tick = std::time::Instant::now();
            }

            // Render UI
            self.render()?;

            if self.should_quit {
                break;
            }
        }

        self.cleanup()?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('p') => self.simulation.toggle_pause(),
            KeyCode::Char('r') => self.simulation.reset(None),
            KeyCode::Up => self.scroll_events_up(),
            KeyCode::Down => self.scroll_events_down(),
            KeyCode::PageUp => self.scroll_events_page_up(),
            KeyCode::PageDown => self.scroll_events_page_down(),
            KeyCode::Home => self.scroll_events_to_top(),
            KeyCode::End => self.scroll_events_to_bottom(),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_events_up(),
            MouseEventKind::ScrollDown => self.scroll_events_down(),
            MouseEventKind::Down(button) => {
                // Handle mouse clicks based on position
                self.handle_mouse_click(mouse.column, mouse.row, button);
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, x: u16, y: u16, button: MouseButton) {
        // Get terminal size to determine click regions
        let size = self.terminal.size().unwrap_or_default();
        
        // Define UI regions (approximate based on layout)
        let header_height = 3;
        let stats_height = 10;
        let controls_height = 4;
        let events_start = header_height + stats_height;
        let events_end = size.height.saturating_sub(controls_height);
        
        match button {
            MouseButton::Left => {
                // Left click actions based on region
                if y >= events_start && y < events_end {
                    // Click in events area - scroll to that position
                    let events = self.simulation.state().get_events();
                    if !events.is_empty() {
                        let click_position = (y - events_start) as usize;
                        let target_offset = click_position.saturating_add(self.event_scroll_offset);
                        let max_offset = events.len().saturating_sub(10);
                        self.event_scroll_offset = target_offset.min(max_offset);
                    }
                } else if y >= size.height.saturating_sub(controls_height) {
                    // Click in controls area - toggle pause
                    self.simulation.toggle_pause();
                }
                // x coordinate could be used for more precise clicking in the future
                let _ = x;
            }
            MouseButton::Right => {
                // Right click to quit
                self.should_quit = true;
            }
            MouseButton::Middle => {
                // Middle click to reset
                self.simulation.reset(None);
                self.event_scroll_offset = 0;
            }
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let renderer = UIRenderer::new();
            renderer.render(f, &self.simulation, self.event_scroll_offset);
        })?;
        Ok(())
    }

    fn scroll_events_up(&mut self) {
        if self.event_scroll_offset > 0 {
            self.event_scroll_offset -= 1;
        }
    }

    fn scroll_events_down(&mut self) {
        let events = self.simulation.state().get_events();
        let max_offset = if events.len() > 10 {
            events.len() - 10
        } else {
            0
        };
        
        if self.event_scroll_offset < max_offset {
            self.event_scroll_offset += 1;
        }
    }

    fn scroll_events_page_up(&mut self) {
        self.event_scroll_offset = self.event_scroll_offset.saturating_sub(10);
    }

    fn scroll_events_page_down(&mut self) {
        let events = self.simulation.state().get_events();
        let max_offset = if events.len() > 10 {
            events.len() - 10
        } else {
            0
        };
        
        self.event_scroll_offset = (self.event_scroll_offset + 10).min(max_offset);
    }

    fn scroll_events_to_top(&mut self) {
        self.event_scroll_offset = 0;
    }

    fn scroll_events_to_bottom(&mut self) {
        let events = self.simulation.state().get_events();
        let max_offset = if events.len() > 10 {
            events.len() - 10
        } else {
            0
        };
        self.event_scroll_offset = max_offset;
    }

    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
