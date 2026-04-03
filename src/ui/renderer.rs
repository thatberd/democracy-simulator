use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Sparkline, Wrap
    },
    Frame,
};
use crate::engine::Simulation;

pub struct UIRenderer;

impl UIRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, simulation: &Simulation, scroll_offset: usize) {
        let state = simulation.state();
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(10), // Stats
                Constraint::Min(6),     // Events
                Constraint::Length(4),  // Controls (increased from 3)
            ])
            .split(f.area());

        self.render_header(f, chunks[0], state);
        self.render_stats(f, chunks[1], state);
        self.render_events(f, chunks[2], state, scroll_offset);
        self.render_controls(f, chunks[3], simulation.is_paused());
    }

    fn render_header(&self, f: &mut Frame, area: Rect, state: &crate::engine::State) {
        let header_text = vec![
            Line::from(vec![
                Span::styled("Democracy Simulator", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled(format!("Seed: {}", state.seed), Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(format!("Tick: {}", state.tick), Style::default().fg(Color::Green)),
            ])
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("Header"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(header, area);
    }

    fn render_stats(&self, f: &mut Frame, area: Rect, state: &crate::engine::State) {
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        // Left side - Basic stats
        let stats_text = vec![
            Line::from(vec![
                Span::styled("Citizens:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {}", state.citizens.len())),
            ]),
            Line::from(vec![
                Span::styled("Avg Ideology:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.3}", state.get_average_ideology_immutable())),
            ]),
            Line::from(vec![
                Span::styled("Avg Happiness:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.3}", state.get_average_happiness_immutable())),
            ]),
            Line::from(vec![
                Span::styled("Avg Trust:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.3}", state.get_average_trust_immutable())),
            ]),
            Line::from(vec![
                Span::styled("Gov Ideology:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.3}", state.government.current_ideology)),
            ]),
            Line::from(vec![
                Span::styled("Term Remaining:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {}", state.government.term_remaining)),
            ]),
        ];

        let stats_para = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Statistics"))
            .style(Style::default().fg(Color::White));

        f.render_widget(stats_para, stats_chunks[0]);

        // Right side - Economy and ideology distribution
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(3),
            ])
            .split(stats_chunks[1]);

        // Economy stats
        let economy_text = vec![
            Line::from(vec![
                Span::styled("GDP:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.3}", state.economy.gdp)),
            ]),
            Line::from(vec![
                Span::styled("Unemployment:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.1}%", state.economy.unemployment * 100.0)),
            ]),
            Line::from(vec![
                Span::styled("Inequality:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {:.1}%", state.economy.inequality * 100.0)),
            ]),
        ];

        let economy_para = Paragraph::new(economy_text)
            .block(Block::default().borders(Borders::ALL).title("Economy"))
            .style(Style::default().fg(Color::White));

        f.render_widget(economy_para, right_chunks[0]);

        // Ideology distribution
        let distribution = state.get_ideology_distribution();
        let distribution_u64: Vec<u64> = distribution.iter().map(|&x| x as u64).collect();
        let max_count = *distribution.iter().max().unwrap_or(&1) as u64;
        
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Ideology Distribution"))
            .data(&distribution_u64)
            .max(max_count)
            .style(Style::default().fg(Color::Blue));

        f.render_widget(sparkline, right_chunks[1]);
    }

    fn render_events(&self, f: &mut Frame, area: Rect, state: &crate::engine::State, scroll_offset: usize) {
        let events = state.get_events();
        
        if events.is_empty() {
            let no_events = Paragraph::new("No events yet...")
                .block(Block::default().borders(Borders::ALL).title("Event Log"))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(no_events, area);
            return;
        }

        // Show events based on scroll offset
        let visible_events = area.height.saturating_sub(2) as usize;
        let start_idx = scroll_offset.min(events.len().saturating_sub(visible_events));
        let end_idx = (start_idx + visible_events).min(events.len());

        let event_items: Vec<ListItem> = if start_idx < events.len() {
            events[start_idx..end_idx]
                .iter()
                .map(|event| ListItem::new(event.as_str()))
                .collect()
        } else {
            Vec::new()
        };

        let events_list = List::new(event_items)
            .block(Block::default().borders(Borders::ALL).title(format!("Event Log ({}-{} of {})", 
                start_idx + 1, end_idx, events.len())))
            .style(Style::default().fg(Color::White));

        f.render_widget(events_list, area);
    }

    fn render_controls(&self, f: &mut Frame, area: Rect, paused: bool) {
        let status = if paused {
            Span::styled("PAUSED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("RUNNING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        };

        let controls_text = vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                status,
                Span::raw("  |  "),
                Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": quit  "),
                Span::styled("p", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": pause/resume  "),
                Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": reset"),
            ]),
            Line::from(vec![
                Span::styled("Event Log: ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": scroll  "),
                Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": page  "),
                Span::styled("Home/End", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": top/bottom"),
            ])
        ];

        let controls_para = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::White));

        f.render_widget(controls_para, area);
    }
}
