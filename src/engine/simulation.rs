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

    fn compute_local_ideology(&mut self, citizen_idx: usize, sample_size: usize) -> f32 {
        if self.state.citizens.len() <= 1 {
            return 0.0;
        }
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for _ in 0..sample_size {
            let sample_idx = loop {
                let idx = self.state.rng.gen_range(0..self.state.citizens.len());
                if idx != citizen_idx {
                    break idx;
                }
            };
            
            sum += self.state.citizens[sample_idx].ideology;
            count += 1;
        }
        
        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }

        let prev_avg_happiness = self.state.get_average_happiness();
        let prev_gdp = self.state.economy.gdp;

        // Update citizen ideologies using local interactions
        let citizen_count = self.state.citizens.len();
        let mut local_averages = Vec::with_capacity(citizen_count);
        
        // Pre-compute local averages for all citizens
        for i in 0..citizen_count {
            let sample_size = self.state.rng.gen_range(3..=8);
            let local_avg = self.compute_local_ideology(i, sample_size);
            local_averages.push(local_avg);
        }
        
        // Apply the updates
        for (i, citizen) in &mut self.state.citizens.iter_mut().enumerate() {
            let noise: f32 = self.state.rng.gen_range(-0.01..0.01);
            
            // INSTABILITY WHEN TRUST IS LOW: Add chaos for low-trust citizens
            let chaos = if citizen.trust_in_government < 0.2 {
                self.state.rng.gen_range(-0.05..0.05) * (1.0 - citizen.trust_in_government)
            } else {
                0.0
            };
            
            citizen.update_ideology_local(local_averages[i], noise, chaos);
        }

        // Add lightweight pairwise citizen interactions
        let interaction_count = (self.state.citizens.len() / 2).max(10); // population size / 2, min 10
        for _ in 0..interaction_count {
            let a_idx = self.state.rng.gen_range(0..self.state.citizens.len());
            let b_idx = self.state.rng.gen_range(0..self.state.citizens.len());
            
            if a_idx != b_idx {
                // Store current ideologies before interaction
                let a_ideology = self.state.citizens[a_idx].ideology;
                let b_ideology = self.state.citizens[b_idx].ideology;
                
                // Apply lightweight interaction (0.01 influence as specified)
                let influence = 0.01;
                self.state.citizens[a_idx].ideology += (b_ideology - a_ideology) * influence;
                self.state.citizens[b_idx].ideology += (a_ideology - b_ideology) * influence;
                
                // Clamp to valid range
                self.state.citizens[a_idx].ideology = self.state.citizens[a_idx].ideology.clamp(-1.0, 1.0);
                self.state.citizens[b_idx].ideology = self.state.citizens[b_idx].ideology.clamp(-1.0, 1.0);
            }
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
        
        // Check for social unrest events based on system conditions
        let avg_trust = self.state.get_average_trust();
        let avg_radicalization = self.state.get_average_radicalization();
        
        // Protests when trust is low and radicalization is high
        if avg_trust < 0.3 && avg_radicalization > 0.5 && self.state.rng.gen::<f32>() < 0.02 {
            self.state.add_event(format!(
                "Tick {}: Mass protests erupt! Trust: {:.2}, Radicalization: {:.2}", 
                self.state.tick, avg_trust, avg_radicalization
            ));
            
            // Protests affect citizen psychology
            for citizen in &mut self.state.citizens {
                if citizen.radicalization > 0.6 {
                    citizen.trust_in_government *= 0.8; // Further reduce trust
                    citizen.radicalization = (citizen.radicalization * 1.1).min(1.0); // Increase radicalization
                }
            }
        }
        
        // Social cohesion events when conditions are good
        if avg_trust > 0.7 && avg_radicalization < 0.3 && self.state.rng.gen::<f32>() < 0.01 {
            self.state.add_event(format!(
                "Tick {}: Period of social harmony! Trust: {:.2}, Radicalization: {:.2}", 
                self.state.tick, avg_trust, avg_radicalization
            ));
            
            // Harmony reduces polarization
            for citizen in &mut self.state.citizens {
                citizen.radicalization *= 0.9;
                citizen.trust_in_government = (citizen.trust_in_government * 1.05).min(1.0);
            }
        }
    }

    fn hold_election(&mut self) {
        let citizen_ideologies: Vec<f32> = self.state.citizens.iter()
            .map(|c| c.ideology)
            .collect();
        
        let old_ideology = self.state.government.current_ideology;
        let new_ideology = self.state.government.hold_election(&citizen_ideologies, &mut self.state.rng);
        
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
