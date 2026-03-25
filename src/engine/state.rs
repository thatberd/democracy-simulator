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
        }
    }
}
