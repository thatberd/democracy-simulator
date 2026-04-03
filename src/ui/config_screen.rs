use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout, Rect, Margin},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::config::SimConfig;

pub struct InputField {
    pub label: String,
    pub value: String,
    pub min: f32,
    pub max: f32,
    pub is_integer: bool,
}

impl InputField {
    pub fn new(label: &str, default_value: &str, min: f32, max: f32, is_integer: bool) -> Self {
        Self {
            label: label.to_string(),
            value: default_value.to_string(),
            min,
            max,
            is_integer,
        }
    }

    pub fn get_f32(&self) -> Result<f32, String> {
        self.value.parse::<f32>()
            .map_err(|_| format!("Invalid number: {}", self.value))
            .map(|v| v.clamp(self.min, self.max))
    }

    pub fn get_u32(&self) -> Result<u32, String> {
        self.value.parse::<u32>()
            .map_err(|_| format!("Invalid number: {}", self.value))
    }
}

pub struct ConfigState {
    pub fields: Vec<InputField>,
    pub selected: usize,
    pub editing: bool,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            fields: vec![
                InputField::new("Citizens", "1000", 100.0, 5000.0, true),
                InputField::new("Inequality", "0.5", 0.0, 1.0, false),
                InputField::new("Trust", "0.5", 0.0, 1.0, false),
                InputField::new("Volatility", "0.5", 0.0, 1.0, false),
            ],
            selected: 0,
            editing: false,
        }
    }

    pub fn from_config(config: &SimConfig) -> Self {
        let mut state = Self::new();
        state.fields[0].value = config.citizens.to_string();
        state.fields[1].value = format!("{:.3}", config.inequality_f32());
        state.fields[2].value = format!("{:.3}", config.trust_f32());
        state.fields[3].value = format!("{:.3}", config.volatility_f32());
        state
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up if !self.editing => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down if !self.editing => {
                if self.selected < self.fields.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                self.editing = !self.editing;
            }
            KeyCode::Tab => {
                if !self.editing {
                    self.selected = (self.selected + 1) % self.fields.len();
                }
            }
            KeyCode::Char('s') if !self.editing => {
                return true; // Signal to start simulation
            }
            KeyCode::Char('q') if !self.editing => {
                std::process::exit(0);
            }
            KeyCode::Backspace if self.editing => {
                self.fields[self.selected].value.pop();
            }
            KeyCode::Char(c) if self.editing => {
                if c.is_ascii_digit() || c == '.' {
                    let field = &mut self.fields[self.selected];
                    if !field.is_integer || c != '.' {
                        field.value.push(c);
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn build_config(&self) -> Result<SimConfig, String> {
        let citizens = self.fields[0].get_u32()?;
        let inequality = self.fields[1].get_f32()?;
        let trust = self.fields[2].get_f32()?;
        let volatility = self.fields[3].get_f32()?;

        Ok(SimConfig {
            citizens,
            initial_inequality: (inequality * 1000.0) as u32,
            initial_trust: (trust * 1000.0) as u32,
            economic_volatility: (volatility * 1000.0) as u32,
        })
    }
}

pub struct ConfigScreen {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: ConfigState,
}

impl ConfigScreen {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: ConfigState::new(),
        })
    }

    pub fn from_config(config: SimConfig) -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: ConfigState::from_config(&config),
        })
    }

    pub fn run(&mut self) -> Result<SimConfig, Box<dyn std::error::Error>> {
        loop {
            self.render()?;

            if event::poll(std::time::Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    // Only process key press events, not release events (fixes Windows double input)
                    if key.kind == KeyEventKind::Press {
                        if self.state.handle_key(key.code) {
                            // User pressed 's' to start
                            break;
                        }
                    }
                }
            }
        }

        let config = self.state.build_config()?;
        self.cleanup()?;
        Ok(config)
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = &self.state;
        self.terminal.draw(|f| {
            let size = f.area();
            
            // Create centered popup
            let popup_area = Self::centered_rect_static(60, 70, size);
            
            f.render_widget(Clear, popup_area);
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Simulation Setup")
                    .title_alignment(Alignment::Center),
                popup_area,
            );

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),  // Title spacing
                    Constraint::Length(10), // Fields
                    Constraint::Length(2),  // Instructions
                ])
                .split(popup_area.inner(Margin::new(1, 1)));

            // Render fields
            for (i, field) in state.fields.iter().enumerate() {
                let is_selected = i == state.selected;
                let is_editing = is_selected && state.editing;
                
                let text = if is_editing {
                    vec![
                        Line::from(vec![
                            Span::styled("> ", ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)),
                            Span::styled(&field.label, ratatui::style::Style::default().fg(ratatui::style::Color::Cyan)),
                            Span::raw(": "),
                            Span::styled(&field.value, ratatui::style::Style::default().fg(ratatui::style::Color::Green)),
                        ])
                    ]
                } else if is_selected {
                    vec![
                        Line::from(vec![
                            Span::styled("> ", ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)),
                            Span::styled(&field.label, ratatui::style::Style::default().fg(ratatui::style::Color::Cyan)),
                            Span::raw(": "),
                            Span::styled(&field.value, ratatui::style::Style::default()),
                        ])
                    ]
                } else {
                    vec![
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(&field.label, ratatui::style::Style::default()),
                            Span::raw(": "),
                            Span::styled(&field.value, ratatui::style::Style::default()),
                        ])
                    ]
                };

                let paragraph = Paragraph::new(text);
                let field_area = Rect {
                    x: chunks[1].x,
                    y: chunks[1].y + i as u16,
                    width: chunks[1].width,
                    height: 1,
                };
                f.render_widget(paragraph, field_area);
            }

            // Instructions
            let instructions = vec![
                Line::from("↑/↓: Select  Enter: Edit  Tab: Next"),
                Line::from("s: Start  q: Quit"),
            ];
            let instructions_paragraph = Paragraph::new(instructions)
                .alignment(Alignment::Center);
            f.render_widget(instructions_paragraph, chunks[2]);
        })?;
        Ok(())
    }

    fn centered_rect(&mut self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    fn centered_rect_static(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
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
