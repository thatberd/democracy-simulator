use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citizen {
    pub ideology: f32,        // -1.0 = far left, 0 = center, 1.0 = far right
    pub happiness: f32,       // 0.0–1.0
    pub trust_in_government: f32, // 0.0–1.0
}

impl Citizen {
    pub fn new(ideology: f32, happiness: f32, trust_in_government: f32) -> Self {
        Self {
            ideology: ideology.clamp(-1.0, 1.0),
            happiness: happiness.clamp(0.0, 1.0),
            trust_in_government: trust_in_government.clamp(0.0, 1.0),
        }
    }

    pub fn update_ideology(&mut self, global_avg_ideology: f32, noise: f32) {
        // Move slightly toward global average with small random noise
        let drift_factor = 0.02;
        self.ideology = self.ideology * (1.0 - drift_factor) + global_avg_ideology * drift_factor + noise;
        self.ideology = self.ideology.clamp(-1.0, 1.0);
    }

    pub fn update_happiness(&mut self, economy: &Economy, government_ideology: f32) {
        let base_happiness = 0.5;
        
        // Economic factors
        let economic_factor = (1.0 - economy.unemployment) * 0.3 + 
                             (1.0 - economy.inequality) * 0.2;
        
        // Political alignment factor
        let ideology_diff = (self.ideology - government_ideology).abs();
        let alignment_factor = (1.0 - ideology_diff) * 0.2;
        
        self.happiness = base_happiness + economic_factor + alignment_factor;
        self.happiness = self.happiness.clamp(0.0, 1.0);
    }

    pub fn update_trust(&mut self, happiness_change: f32, economy_change: f32) {
        let trust_change = happiness_change * 0.3 + economy_change * 0.2;
        self.trust_in_government += trust_change;
        self.trust_in_government = self.trust_in_government.clamp(0.0, 1.0);
    }
}

use crate::engine::Economy;
