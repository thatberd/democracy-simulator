use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::engine::Simulation;
use crate::config::SimConfig;
use super::renderer::UIRenderer;

pub struct App {
    simulation: Simulation,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    should_quit: bool,
    event_scroll_offset: usize,
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
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_tick = std::time::Instant::now();
        let tick_duration = std::time::Duration::from_millis(100); // 10 FPS

        loop {
            // Handle input with single event read
            if event::poll(std::time::Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    // Only process key press events, not release events (fixes Windows double input)
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key);
                    }
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
