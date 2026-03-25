use crate::engine::State;
use rand::Rng;

pub struct Simulation {
    state: State,
    paused: bool,
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        Self {
            state: State::new(seed),
            paused: false,
        }
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }

        let prev_avg_happiness = self.state.get_average_happiness();
        let prev_gdp = self.state.economy.gdp;

        // Update citizen ideologies
        let global_avg_ideology = self.state.get_average_ideology();
        for citizen in &mut self.state.citizens {
            let noise: f32 = self.state.rng.gen_range(-0.01..0.01);
            citizen.update_ideology(global_avg_ideology, noise);
        }

        // Update citizen happiness
        for citizen in &mut self.state.citizens {
            citizen.update_happiness(&self.state.economy, self.state.government.current_ideology);
        }

        // Update citizen trust
        let avg_happiness_change = self.state.get_average_happiness() - prev_avg_happiness;
        let gdp_change = self.state.economy.gdp - prev_gdp;
        for citizen in &mut self.state.citizens {
            citizen.update_trust(avg_happiness_change, gdp_change);
        }

        // Update economy
        let economic_drift: f32 = self.state.rng.gen_range(-0.02..0.02);
        self.state.economy.update(self.state.government.current_ideology, economic_drift);

        // Random events
        self.handle_random_events();

        // Check for elections
        if self.state.government.is_election_due() {
            self.hold_election();
        }

        // Update government term
        self.state.government.update_term();

        self.state.tick += 1;
    }

    fn handle_random_events(&mut self) {
        let event_chance: f32 = self.state.rng.gen();
        
        if event_chance < 0.005 { // 0.5% chance per tick
            self.state.economy.trigger_crisis();
            self.state.add_event(format!("Tick {}: Economic crisis struck!", self.state.tick));
        } else if event_chance < 0.01 { // Additional 0.5% chance
            self.state.economy.trigger_boom();
            self.state.add_event(format!("Tick {}: Economic boom!", self.state.tick));
        }
    }

    fn hold_election(&mut self) {
        let citizen_ideologies: Vec<f32> = self.state.citizens.iter()
            .map(|c| c.ideology)
            .collect();
        
        let old_ideology = self.state.government.current_ideology;
        let new_ideology = self.state.government.hold_election(&citizen_ideologies);
        
        let direction = if new_ideology > old_ideology {
            "right"
        } else if new_ideology < old_ideology {
            "left"
        } else {
            "no change"
        };
        
        self.state.add_event(format!(
            "Tick {}: Election held. Government shifted {} ({} → {})",
            self.state.tick, direction, 
            format_ideology(old_ideology),
            format_ideology(new_ideology)
        ));
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn reset(&mut self, new_seed: Option<u64>) {
        let seed = new_seed.unwrap_or_else(|| {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen()
        });
        self.state = State::new(seed);
        self.paused = false;
    }
}

fn format_ideology(ideology: f32) -> String {
    if ideology < -0.5 {
        format!("Far Left ({:.2})", ideology)
    } else if ideology < -0.1 {
        format!("Left ({:.2})", ideology)
    } else if ideology <= 0.1 {
        format!("Center ({:.2})", ideology)
    } else if ideology <= 0.5 {
        format!("Right ({:.2})", ideology)
    } else {
        format!("Far Right ({:.2})", ideology)
    }
}
