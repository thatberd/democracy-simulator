use crate::engine::State;
use crate::config::SimConfig;
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

    pub fn new_with_config(seed: u64, config: SimConfig) -> Self {
        Self {
            state: State::new_with_config(seed, config),
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
                let mut polarization_factor = 1.5; // Base polarization for low trust
                
                // ESCALATION EFFECTS: Prolonged hardship amplifies instability
                if self.state.hardship_duration > 50 {
                    polarization_factor *= 1.2; // 20% more instability after 50 ticks of hardship
                }
                if self.state.hardship_duration > 100 {
                    polarization_factor *= 1.5; // 50% more instability after 100 ticks of hardship
                }
                
                self.state.rng.gen_range(-0.05..0.05) * (1.0 - citizen.trust_in_government) * polarization_factor
            } else {
                0.0
            };
            
            // IDEOLOGICAL DRIFT UNDER LOW HAPPINESS: misery creates ideological movement
            let happiness_drift = if citizen.happiness < 0.2 {
                let drift_strength = (0.2 - citizen.happiness) * 0.25; // Scale drift based on how low happiness is
                if self.state.rng.gen::<f32>() < 0.3 { // 30% chance per tick when very unhappy
                    self.state.rng.gen_range(-drift_strength..drift_strength)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            citizen.update_ideology_local(local_averages[i], noise, chaos, happiness_drift);
        }

        // Add echo chamber citizen interactions
        let interaction_count = (self.state.citizens.len() / 2).max(10); // population size / 2, min 10
        for _ in 0..interaction_count {
            let a_idx = self.state.rng.gen_range(0..self.state.citizens.len());
            let b_idx = self.state.rng.gen_range(0..self.state.citizens.len());
            
            if a_idx != b_idx {
                // Clone citizen data for interaction to avoid borrowing issues
                let citizen_b_clone = self.state.citizens[b_idx].clone();
                let citizen_a_clone = self.state.citizens[a_idx].clone();
                
                // Use the new echo chamber interaction method
                self.state.citizens[a_idx].interact_with(&citizen_b_clone);
                self.state.citizens[b_idx].interact_with(&citizen_a_clone);
            }
        }

        // Update citizen happiness
        for citizen in &mut self.state.citizens {
            citizen.update_happiness(&self.state.economy, self.state.government.current_ideology);
        }

        // Update citizen trust
        let avg_happiness_change = self.state.get_average_happiness() - prev_avg_happiness;
        let gdp_change = self.state.economy.gdp - prev_gdp;
        
        // PHASE TRANSITION: Track hardship duration
        let avg_happiness = self.state.get_average_happiness();
        if avg_happiness < 0.2 {
            self.state.hardship_duration += 1;
        } else {
            self.state.hardship_duration = 0;
        }
        
        for citizen in &mut self.state.citizens {
            citizen.update_trust(avg_happiness_change, gdp_change);
            
            // TRUST DECAY UNDER PROLONGED HARDSHIP: Chronic misery erodes trust
            if avg_happiness < 0.2 {
                let trust_decay = 0.01 * (1.0 + self.state.hardship_duration as f32 * 0.001); // Scale with duration
                citizen.trust_in_government = (citizen.trust_in_government - trust_decay).max(0.0);
            }
            
            // Apply ongoing reform effects with RECOVERY ACCELERATION
            if self.state.reform_active {
                let mut trust_boost = 0.005 * self.state.reform_strength;
                let mut happiness_boost = 0.003 * self.state.reform_strength;
                
                // RECOVERY ACCELERATION: Stronger bounce-back after prolonged hardship
                if self.state.hardship_duration > 100 {
                    trust_boost *= 2.0;
                    happiness_boost *= 2.0;
                }
                
                citizen.trust_in_government = (citizen.trust_in_government + trust_boost).min(1.0);
                citizen.happiness = (citizen.happiness + happiness_boost).min(1.0);
                citizen.radicalization *= 1.0 - 0.01 * self.state.reform_strength; // Gradual deradicalization
            }
        }

        // Update citizen memory for trend-based behavior
        for citizen in &mut self.state.citizens {
            citizen.update_memory();
        }

        // Apply inequality-driven polarization
        for citizen in &mut self.state.citizens {
            citizen.increase_polarization_from_inequality(self.state.economy.inequality);
        }

        // Apply natural stabilization drift to prevent permanent deadlock
        for citizen in &mut self.state.citizens {
            // PREVENT PERMANENT CENTER LOCK: Add weak long-term drift
            let center_drift = if self.state.rng.gen::<f32>() < 0.1 { // 10% chance per tick
                self.state.rng.gen_range(-0.01..0.01) // Weak drift to avoid perfect equilibrium
            } else {
                0.0
            };
            citizen.apply_natural_stabilization_drift(center_drift);
            
            // POLARIZATION FROM CHRONIC STRESS: Prolonged hardship pushes society to extremes
            if self.state.hardship_duration > 80 {
                let polarization_strength = 1.02; // 2% increase per tick
                citizen.ideology *= polarization_strength;
                citizen.ideology = citizen.ideology.clamp(-1.0, 1.0);
            }
        }

        // Update economy with policy lag
        let economic_drift: f32 = self.state.rng.gen_range(-0.02..0.02);
        let lagged_ideology = self.state.government.get_lagged_ideology();
        self.state.economy.update(lagged_ideology, economic_drift);

        // Random events
        self.handle_random_events();

        // Update reform state
        self.state.update_reform();

        // Check for elections
        if self.state.government.is_election_due() {
            self.hold_election();
        }

        // Update government term and policy queue
        self.state.government.update_term();
        self.state.government.update_policy_queue();

        self.state.tick += 1;
    }

    fn handle_random_events(&mut self) {
        let event_chance: f32 = self.state.rng.gen();
        
        // INSTABILITY PRESSURE: Calculate system instability to increase event probability
        let avg_trust = self.state.get_average_trust();
        let avg_happiness = self.state.get_average_happiness();
        let instability = (1.0 - avg_trust) + (1.0 - avg_happiness); // 0.0 to 2.0
        let mut instability_multiplier = 1.0 + instability * 0.5; // 1.0 to 2.0 multiplier
        
        // ESCALATION EFFECTS: Prolonged hardship increases instability
        if self.state.hardship_duration > 100 {
            instability_multiplier *= 1.5; // 50% more instability after 100 ticks of hardship
        }
        
        // Apply instability to event chances
        let adjusted_event_chance = event_chance * instability_multiplier;
        
        // Economic crisis events with narrative context and cooldowns
        if adjusted_event_chance < 0.005 && !self.state.is_event_on_cooldown("crisis", 30) { // 0.5% chance per tick, 30 tick cooldown
            let gdp_trend = self.state.economy.gdp - self.state.economy.previous_gdp;
            let reason = if gdp_trend < -0.05 {
                "triggered by sharp GDP decline"
            } else if self.state.economy.unemployment > 0.4 {
                "amid rising unemployment"
            } else if self.state.economy.inequality > 0.8 {
                "due to extreme inequality"
            } else {
                "from economic instability"
            };
            
            self.state.economy.trigger_crisis();
            self.state.last_crisis_tick = self.state.tick;
            self.state.add_event(format!(
                "Tick {}: Economic crisis struck {}! GDP: {:.2}, Unemployment: {:.2}", 
                self.state.tick, reason, self.state.economy.gdp, self.state.economy.unemployment
            ));
        } else if adjusted_event_chance < 0.01 && !self.state.is_event_on_cooldown("crisis", 30) { // Additional 0.5% chance
            let reason = if self.state.economy.gdp > 1.2 {
                "driven by strong growth"
            } else if self.state.economy.unemployment < 0.1 {
                "with low employment"
            } else {
                "from favorable conditions"
            };
            
            self.state.economy.trigger_boom();
            self.state.add_event(format!(
                "Tick {}: Economic boom {}! GDP: {:.2}, Unemployment: {:.2}", 
                self.state.tick, reason, self.state.economy.gdp, self.state.economy.unemployment
            ));
        }
        
        // Check for social unrest events based on system conditions
        let avg_trust = self.state.get_average_trust();
        let avg_happiness = self.state.get_average_happiness();
        let avg_radicalization = self.state.get_average_radicalization();
        
        // Protests with fatigue and cooldowns
        if avg_trust < 0.2 && avg_happiness < 0.3 && !self.state.is_event_on_cooldown("protest", 20) {
            let protest_fatigue = self.state.get_protest_fatigue();
            let protest_chance = 0.02 * protest_fatigue;
            
            if self.state.rng.gen::<f32>() < protest_chance {
                let trigger = if avg_happiness < 0.1 {
                    "mass despair"
                } else if avg_trust < 0.1 {
                    "complete government distrust"
                } else {
                    "widespread suffering"
                };
                
                self.state.add_event(format!(
                    "Tick {}: Mass protests erupt from {}! Trust: {:.2}, Happiness: {:.2}, Radicalization: {:.2}", 
                    self.state.tick, trigger, avg_trust, avg_happiness, avg_radicalization
                ));
                
                // Update protest tracking
                self.state.last_protest_tick = self.state.tick;
                self.state.update_protest_history(true);
                
                // Protests affect citizen psychology and decrease stability
                for citizen in &mut self.state.citizens {
                    if citizen.radicalization > 0.6 {
                        citizen.trust_in_government *= 0.8; // Further reduce trust
                        citizen.radicalization = (citizen.radicalization * 1.1).min(1.0); // Increase radicalization
                        citizen.ideology *= 1.05; // Increase ideological volatility (push toward extremes)
                        citizen.ideology = citizen.ideology.clamp(-1.0, 1.0);
                    }
                }
            } else {
                self.state.update_protest_history(false);
            }
        } else {
            self.state.update_protest_history(false);
        }
        
        // LATENT UNREST TRIGGER: Even without polarization, bad conditions should trigger events
        if avg_happiness < 0.1 && avg_trust < 0.1 && !self.state.is_event_on_cooldown("protest", 15) {
            let latent_unrest_chance = 0.05; // 5% chance per tick when conditions are catastrophic
            
            if self.state.rng.gen::<f32>() < latent_unrest_chance {
                self.state.add_event(format!(
                    "Tick {}: Spontaneous uprising from catastrophic conditions! Trust: {:.3}, Happiness: {:.3}", 
                    self.state.tick, avg_trust, avg_happiness
                ));
                
                // Update protest tracking
                self.state.last_protest_tick = self.state.tick;
                self.state.update_protest_history(true);
                
                // Severe unrest effects - affects all citizens
                for citizen in &mut self.state.citizens {
                    citizen.trust_in_government *= 0.7; // Major trust reduction
                    citizen.happiness *= 0.8; // Happiness drops further
                    citizen.radicalization = (citizen.radicalization * 1.2).min(1.0); // Increase radicalization
                    
                    // Push ideologies toward extremes randomly
                    if self.state.rng.gen::<f32>() < 0.5 {
                        citizen.ideology = (citizen.ideology * 1.1).min(1.0);
                    } else {
                        citizen.ideology = (citizen.ideology * 1.1).max(-1.0);
                    }
                }
            } else {
                self.state.update_protest_history(false);
            }
        } else {
            self.state.update_protest_history(false);
        }
        
        // Reform / Recovery events with cooldowns
        let reform_conditions = (avg_trust < 0.2 && (avg_radicalization > 0.6 || self.state.economy.inequality > 0.7)) 
            || (self.state.economy.inequality > 0.6 && self.state.rng.gen::<f32>() < 0.01);
            
        if reform_conditions && !self.state.is_event_on_cooldown("reform", 40) {
            let reform_context = if avg_trust < 0.1 {
                "amid complete system breakdown"
            } else if self.state.economy.inequality > 0.8 {
                "driven by extreme economic disparity"
            } else if avg_radicalization > 0.7 {
                "from overwhelming social polarization"
            } else {
                "through growing public discontent"
            };
            
            self.state.add_event(format!(
                "Tick {}: Reform movement emerges {} - Restoring partial trust and stabilizing society", 
                self.state.tick, reform_context
            ));
            
            // Update reform tracking - start persistent reform
            self.state.last_reform_tick = self.state.tick;
            self.state.start_reform(30, 2.0); // 30 ticks duration, 2x strength multiplier
            
            // Initial reform effects
            for citizen in &mut self.state.citizens {
                let trust_gain = 0.2 * self.state.reform_strength;
                let happiness_gain = 0.1 * self.state.reform_strength;
                citizen.trust_in_government = (citizen.trust_in_government + trust_gain).min(1.0);
                citizen.happiness = (citizen.happiness + happiness_gain).min(1.0);
                citizen.radicalization *= 0.8;
            }
            
            // Slightly reduce inequality and stabilize economy
            self.state.economy.inequality *= 0.9;
            self.state.economy.gdp *= 1.05;
            self.state.economy.unemployment *= 0.95;
        }
        
        // Reform/recovery events when conditions improve after hardship
        if avg_trust > 0.6 && avg_happiness > 0.5 && !self.state.is_event_on_cooldown("reform", 50) {
            let context = if avg_trust > 0.8 {
                "high public confidence"
            } else if avg_happiness > 0.7 {
                "general prosperity"
            } else {
                "improving conditions"
            };
            
            self.state.add_event(format!(
                "Tick {}: Period of reform and recovery begins with {}! Trust: {:.2}, Happiness: {:.2}", 
                self.state.tick, context, avg_trust, avg_happiness
            ));
            
            // Recovery effects
            for citizen in &mut self.state.citizens {
                citizen.trust_in_government = (citizen.trust_in_government * 1.1).min(1.0);
                citizen.radicalization *= 0.9;
                citizen.happiness = (citizen.happiness * 1.05).min(1.0);
            }
            
            // Small economic boost from recovery
            self.state.economy.gdp *= 1.05;
            self.state.economy.unemployment *= 0.95;
        }
        
        // Social cohesion events when conditions are good
        if avg_trust > 0.7 && avg_radicalization < 0.3 && !self.state.is_event_on_cooldown("reform", 60) {
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
        
        // PHASE TRANSITION TRIGGERS: Force major system changes after prolonged hardship
        if self.state.hardship_duration > 150 {
            let roll = self.state.rng.gen_range(0.0..1.0);
            
            if roll < 0.33 {
                // TRIGGER COLLAPSE
                self.trigger_collapse();
            } else if roll < 0.66 {
                // TRIGGER MAJOR REFORM
                self.trigger_major_reform();
            } else {
                // TRIGGER POLARIZATION
                self.trigger_polarization();
            }
        }
    }
    
    fn trigger_collapse(&mut self) {
        self.state.add_event(format!(
            "Tick {}: SYSTEM COLLAPSE! Prolonged hardship ({} ticks) triggered total societal breakdown!", 
            self.state.tick, self.state.hardship_duration
        ));
        
        // Collapse effects: severe trust loss, happiness crash, radicalization spike
        for citizen in &mut self.state.citizens {
            citizen.trust_in_government *= 0.1; // Near-total trust loss
            citizen.happiness *= 0.3; // Happiness crashes
            citizen.radicalization = (citizen.radicalization * 2.0).min(1.0); // Radicalization spikes
            
            // Push ideologies to random extremes
            if self.state.rng.gen::<f32>() < 0.5 {
                citizen.ideology = (citizen.ideology * 1.5).min(1.0);
            } else {
                citizen.ideology = (citizen.ideology * 1.5).max(-1.0);
            }
        }
        
        // Economic collapse
        self.state.economy.gdp *= 0.5;
        self.state.economy.unemployment = (self.state.economy.unemployment * 2.0).min(1.0);
        self.state.economy.inequality = (self.state.economy.inequality * 1.3).min(1.0);
        
        // HARDSHIP RESET: Reduce hardship after major transition
        self.state.hardship_duration = (self.state.hardship_duration / 3).max(0);
    }
    
    fn trigger_major_reform(&mut self) {
        self.state.add_event(format!(
            "Tick {}: MAJOR REFORM! Prolonged hardship ({} ticks) forces systemic transformation!", 
            self.state.tick, self.state.hardship_duration
        ));
        
        // Start powerful reform with extended duration
        self.state.start_reform(60, 3.0); // 60 ticks, 3x strength
        
        // Major reform effects: significant trust and happiness recovery
        for citizen in &mut self.state.citizens {
            let trust_gain = 0.4 * self.state.reform_strength;
            let happiness_gain = 0.3 * self.state.reform_strength;
            citizen.trust_in_government = (citizen.trust_in_government + trust_gain).min(1.0);
            citizen.happiness = (citizen.happiness + happiness_gain).min(1.0);
            citizen.radicalization *= 0.5; // Major deradicalization
            
            // Move ideologies toward center
            citizen.ideology *= 0.7;
        }
        
        // Economic recovery
        self.state.economy.gdp *= 1.3;
        self.state.economy.unemployment *= 0.7;
        self.state.economy.inequality *= 0.8;
        
        // HARDSHIP RESET: Reduce hardship after major transition
        self.state.hardship_duration = (self.state.hardship_duration / 2).max(0);
    }
    
    fn trigger_polarization(&mut self) {
        self.state.add_event(format!(
            "Tick {}: MASS POLARIZATION! Prolonged hardship ({} ticks) shatters social cohesion!", 
            self.state.tick, self.state.hardship_duration
        ));
        
        // Polarization effects: split society into factions
        for citizen in &mut self.state.citizens {
            citizen.trust_in_government *= 0.6; // Moderate trust loss
            citizen.radicalization = (citizen.radicalization * 1.5).min(1.0); // Increased radicalization
            
            // Force citizens to ideological extremes
            if self.state.rng.gen::<f32>() < 0.5 {
                // Push to left
                citizen.ideology = (citizen.ideology - 0.3).max(-1.0);
            } else {
                // Push to right
                citizen.ideology = (citizen.ideology + 0.3).min(1.0);
            }
        }
        
        // Economic stress from polarization
        self.state.economy.gdp *= 0.8;
        self.state.economy.inequality = (self.state.economy.inequality * 1.2).min(1.0);
        
        // HARDSHIP RESET: Reduce hardship after major transition
        self.state.hardship_duration = (self.state.hardship_duration / 2).max(0);
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
        
        // Add narrative context to election results
        let avg_trust = self.state.get_average_trust();
        let avg_happiness = self.state.get_average_happiness();
        let context = if avg_trust < 0.3 {
            "amid public distrust"
        } else if avg_happiness < 0.3 {
            "during widespread hardship"
        } else if avg_trust > 0.7 {
            "with strong public confidence"
        } else if avg_happiness > 0.7 {
            "in prosperous times"
        } else {
            "in uncertain times"
        };
        
        let magnitude = (new_ideology - old_ideology).abs();
        let magnitude_desc = if magnitude > 0.3 {
            "dramatic"
        } else if magnitude > 0.1 {
            "significant"
        } else if magnitude > 0.05 {
            "moderate"
        } else {
            "slight"
        };
        
        self.state.add_event(format!(
            "Tick {}: {} {} shift {} - Government moved from {} to {}",
            self.state.tick, magnitude_desc, context, direction,
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
