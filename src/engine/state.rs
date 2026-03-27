use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::engine::{Citizen, Economy, Government};

#[derive(Debug, Clone)]
pub struct State {
    pub citizens: Vec<Citizen>,
    pub economy: Economy,
    pub government: Government,
    pub tick: u64,
    pub seed: u64,
    pub rng: StdRng, // Note: StdRng doesn't serialize, but we'll reconstruct it
    events: Vec<String>,
    // Event tracking for cooldowns and fatigue
    pub last_protest_tick: u64,
    pub last_reform_tick: u64,
    pub last_crisis_tick: u64,
    pub protest_history: Vec<bool>, // Track recent protests for fatigue
    // Reform tracking for persistent effects
    pub reform_active: bool,
    pub reform_duration: u64, // Ticks remaining for reform effects
    pub reform_strength: f32, // Strength multiplier for reform effects
    // Phase transition tracking
    pub hardship_duration: u64, // Ticks of prolonged low happiness
}

impl Default for State {
    fn default() -> Self {
        Self::new(0)
    }
}

impl State {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        
        // Generate 500-2000 citizens
        let citizen_count = rng.gen_range(500..=2000);
        let mut citizens = Vec::with_capacity(citizen_count);
        
        for _ in 0..citizen_count {
            // Normal distribution around 0 with variance
            let ideology = Self::normal_distribution(&mut rng, 0.0, 0.3).clamp(-1.0, 1.0);
            let happiness = rng.gen_range(0.3..0.8);
            let trust = rng.gen_range(0.4..0.7);
            
            citizens.push(Citizen::new(ideology, happiness, trust));
        }
        
        // Initialize economy
        let gdp = rng.gen_range(0.8..1.2);
        let unemployment = rng.gen_range(0.05..0.15);
        let inequality = rng.gen_range(0.2..0.5);
        let economy = Economy::new(gdp, unemployment, inequality);
        
        // Initialize government with average citizen ideology
        let avg_ideology = citizens.iter().map(|c| c.ideology).sum::<f32>() / citizens.len() as f32;
        let government = Government::new(avg_ideology, 50);
        
        Self {
            citizens,
            economy,
            government,
            tick: 0,
            seed,
            rng,
            events: Vec::new(),
            last_protest_tick: 0,
            last_reform_tick: 0,
            last_crisis_tick: 0,
            protest_history: vec![false; 20], // Track last 20 ticks for fatigue
            reform_active: false,
            reform_duration: 0,
            reform_strength: 0.0,
            hardship_duration: 0,
        }
    }

    // Helper method to generate normal distribution
    fn normal_distribution(rng: &mut StdRng, mean: f32, std_dev: f32) -> f32 {
        // Box-Muller transform
        let u1: f32 = rng.gen();
        let u2: f32 = rng.gen();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        mean + z0 * std_dev
    }

    pub fn add_event(&mut self, event: String) {
        self.events.push(event);
        // Keep only last 100 events
        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }

    pub fn update_protest_history(&mut self, had_protest: bool) {
        self.protest_history.push(had_protest);
        if self.protest_history.len() > 20 {
            self.protest_history.remove(0);
        }
    }

    pub fn get_protest_fatigue(&self) -> f32 {
        let recent_protests = self.protest_history.iter().filter(|&&x| x).count();
        if recent_protests > 5 {
            0.3 // High fatigue
        } else if recent_protests > 2 {
            0.6 // Moderate fatigue
        } else {
            1.0 // No fatigue
        }
    }

    pub fn is_event_on_cooldown(&self, event_type: &str, cooldown_ticks: u64) -> bool {
        let last_event = match event_type {
            "protest" => self.last_protest_tick,
            "reform" => self.last_reform_tick,
            "crisis" => self.last_crisis_tick,
            _ => 0,
        };
        self.tick - last_event < cooldown_ticks
    }

    pub fn start_reform(&mut self, duration: u64, strength: f32) {
        self.reform_active = true;
        self.reform_duration = duration;
        self.reform_strength = strength;
    }

    pub fn update_reform(&mut self) {
        if self.reform_active {
            if self.reform_duration > 0 {
                self.reform_duration -= 1;
                // Gradually decay reform strength
                self.reform_strength *= 0.98;
            } else {
                // Reform expired
                self.reform_active = false;
                self.reform_strength = 0.0;
            }
        }
    }

    pub fn get_events(&self) -> &[String] {
        &self.events
    }

    pub fn get_average_ideology(&self) -> f32 {
        if self.citizens.is_empty() {
            0.0
        } else {
            self.citizens.iter().map(|c| c.ideology).sum::<f32>() / self.citizens.len() as f32
        }
    }

    pub fn get_average_happiness(&self) -> f32 {
        if self.citizens.is_empty() {
            0.0
        } else {
            self.citizens.iter().map(|c| c.happiness).sum::<f32>() / self.citizens.len() as f32
        }
    }

    pub fn get_average_trust(&self) -> f32 {
        if self.citizens.is_empty() {
            0.0
        } else {
            self.citizens.iter().map(|c| c.trust_in_government).sum::<f32>() / self.citizens.len() as f32
        }
    }

    pub fn get_average_radicalization(&self) -> f32 {
        if self.citizens.is_empty() {
            0.0
        } else {
            self.citizens.iter().map(|c| c.radicalization).sum::<f32>() / self.citizens.len() as f32
        }
    }

    pub fn get_ideology_distribution(&self) -> [usize; 10] {
        let mut distribution = [0; 10];
        for citizen in &self.citizens {
            let index = ((citizen.ideology + 1.0) * 5.0) as usize;
            let index = index.min(9);
            distribution[index] += 1;
        }
        distribution
    }

    // For serialization purposes, we need to reconstruct the RNG
    pub fn serialize_state(&self) -> Result<String, serde_json::Error> {
        let serializable = SerializableState::from(self);
        serde_json::to_string(&serializable)
    }

    pub fn deserialize_state(json: &str) -> Result<Self, serde_json::Error> {
        let serializable: SerializableState = serde_json::from_str(json)?;
        Ok(serializable.into())
    }
}

// Helper struct for serialization (without RNG)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableState {
    citizens: Vec<Citizen>,
    economy: Economy,
    government: Government,
    tick: u64,
    seed: u64,
    events: Vec<String>,
    last_protest_tick: u64,
    last_reform_tick: u64,
    last_crisis_tick: u64,
    protest_history: Vec<bool>,
    reform_active: bool,
    reform_duration: u64,
    reform_strength: f32,
    hardship_duration: u64,
}

impl From<&State> for SerializableState {
    fn from(state: &State) -> Self {
        Self {
            citizens: state.citizens.clone(),
            economy: state.economy.clone(),
            government: state.government.clone(),
            tick: state.tick,
            seed: state.seed,
            events: state.events.clone(),
            last_protest_tick: state.last_protest_tick,
            last_reform_tick: state.last_reform_tick,
            last_crisis_tick: state.last_crisis_tick,
            protest_history: state.protest_history.clone(),
            reform_active: state.reform_active,
            reform_duration: state.reform_duration,
            reform_strength: state.reform_strength,
            hardship_duration: state.hardship_duration,
        }
    }
}

impl From<SerializableState> for State {
    fn from(serializable: SerializableState) -> Self {
        let rng = StdRng::seed_from_u64(serializable.seed);
        Self {
            citizens: serializable.citizens,
            economy: serializable.economy,
            government: serializable.government,
            tick: serializable.tick,
            seed: serializable.seed,
            rng,
            events: serializable.events,
            last_protest_tick: serializable.last_protest_tick,
            last_reform_tick: serializable.last_reform_tick,
            last_crisis_tick: serializable.last_crisis_tick,
            protest_history: serializable.protest_history,
            reform_active: serializable.reform_active,
            reform_duration: serializable.reform_duration,
            reform_strength: serializable.reform_strength,
            hardship_duration: 0, // Reset on deserialize
        }
    }
}
