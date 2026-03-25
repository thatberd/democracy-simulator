use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::engine::Simulation;
use super::renderer::UIRenderer;

pub struct App {
    simulation: Simulation,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    should_quit: bool,
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
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_tick = std::time::Instant::now();
        let tick_duration = std::time::Duration::from_millis(100); // 10 FPS

        loop {
            // Handle input
            while event::poll(std::time::Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
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
            _ => {}
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let renderer = UIRenderer::new();
            renderer.render(f, &self.simulation);
        })?;
        Ok(())
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
