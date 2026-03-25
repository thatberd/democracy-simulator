use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citizen {
    pub ideology: f32,        // -1.0 = far left, 0 = center, 1.0 = far right
    pub happiness: f32,       // 0.0–1.0
    pub trust_in_government: f32, // 0.0–1.0
    pub radicalization: f32,  // 0.0–1.0, how extreme/committed to ideology
    pub previous_ideology: f32, // for tracking change rate
    pub previous_happiness: f32,
    pub previous_trust: f32,
}

impl Citizen {
    pub fn new(ideology: f32, happiness: f32, trust_in_government: f32) -> Self {
        let ideology = ideology.clamp(-1.0, 
            1.0);
        let happiness = happiness.clamp(0.0, 1.0);
        let trust = trust_in_government.clamp(0.0, 1.0);
        
        Self {
            ideology,
            happiness,
            trust_in_government: trust,
            radicalization: 0.2, // Start with low radicalization
            previous_ideology: ideology,
            previous_happiness: happiness,
            previous_trust: trust,
        }
    }

    pub fn update_ideology_local(&mut self, local_avg_ideology: f32, noise: f32, chaos: f32) {
        // Store previous value for change tracking
        self.previous_ideology = self.ideology;
        
        // Nonlinear influence based on trust and happiness
        let base_influence = 0.03;
        let trust_factor = 1.0 - self.trust_in_government; // Low trust = more influence from peers
        let happiness_factor = if self.happiness < 0.3 { 1.5 } else { 1.0 }; // Unhappy people more susceptible
        
        let influence_strength = base_influence * trust_factor * happiness_factor;
        
        // Apply nonlinear transformation using tanh for saturation
        let ideology_diff = local_avg_ideology - self.ideology;
        let nonlinear_diff = ideology_diff.tanh() * 0.5; // Saturates at extreme differences
        
        // IDEOLOGICAL REPULSION: Push away from local average if too far
        let distance = (self.ideology - local_avg_ideology).abs();
        let repulsion_effect = if distance > 0.5 {
            // Push further away when very different from neighbors
            (self.ideology - local_avg_ideology) * 0.05
        } else {
            0.0
        };
        
        self.ideology += nonlinear_diff * influence_strength + noise + repulsion_effect + chaos;
        self.ideology = self.ideology.clamp(-1.0, 1.0);
        
        // Update radicalization based on extremeness and social conditions
        self.update_radicalization();
    }
    
    pub fn interact_with(&mut self, other: &Citizen) {
        // Citizen-to-citizen interaction with mutual influence
        let ideology_diff = other.ideology - self.ideology;
        let distance = ideology_diff.abs();
        
        // Influence strength decreases with ideological distance
        let base_strength = 0.02;
        let distance_factor = (-distance * 3.0).exp(); // Exponential decay with distance
        
        // Faction bonus: similar ideologies influence each other more
        let faction_bonus = if distance < 0.2 { 1.5 } else { 0.5 };
        
        let influence = ideology_diff * base_strength * distance_factor * faction_bonus;
        
        // Apply influence with saturation
        self.ideology += influence.tanh() * 0.1;
        self.ideology = self.ideology.clamp(-1.0, 1.0);
    }
    
    fn update_radicalization(&mut self) {
        // Radicalization increases with extremeness and low trust
        let extremeness = self.ideology.abs();
        let trust_factor = 1.0 - self.trust_in_government;
        let happiness_factor = if self.happiness < 0.3 { 1.2 } else { 0.8 };
        
        let target_radicalization = extremeness * trust_factor * happiness_factor;
        
        // Gradual change toward target
        self.radicalization += (target_radicalization - self.radicalization) * 0.05;
        self.radicalization = self.radicalization.clamp(0.0, 1.0);
    }

    pub fn update_happiness(&mut self, economy: &Economy, government_ideology: f32) {
        // Store previous value
        self.previous_happiness = self.happiness;
        
        let base_happiness = 0.4;
        
        // Economic factors with nonlinear effects
        let unemployment_impact = if economy.unemployment > 0.3 {
            // Threshold effect: high unemployment causes disproportionate unhappiness
            -(economy.unemployment - 0.3).powi(2) * 2.0
        } else {
            -(economy.unemployment * 0.3)
        };
        
        let inequality_impact = if economy.inequality > 0.6 {
            // High inequality triggers strong negative response
            -(economy.inequality - 0.6).powi(3) * 3.0
        } else {
            -(economy.inequality * 0.2)
        };
        
        // Political alignment with nonlinear amplification for extremists
        let ideology_diff = (self.ideology - government_ideology).abs();
        let alignment_factor = if ideology_diff > 0.5 {
            // Extremists experience stronger dissatisfaction
            -(ideology_diff * 0.4 * (1.0 + self.radicalization))
        } else {
            (1.0 - ideology_diff) * 0.2
        };
        
        // Trust creates feedback loop
        let trust_bonus = self.trust_in_government * 0.1;
        
        let mut new_happiness = base_happiness + unemployment_impact + inequality_impact + alignment_factor + trust_bonus;
        
        // Apply sigmoid transformation for saturation effects
        new_happiness = (new_happiness * 4.0 - 2.0).tanh() * 0.5 + 0.5;
        
        self.happiness = new_happiness.clamp(0.0, 1.0);
    }

    pub fn update_trust(&mut self, happiness_change: f32, economy_change: f32) {
        // Store previous value
        self.previous_trust = self.trust_in_government;
        
        // Nonlinear response to changes
        let happiness_impact = if happiness_change < -0.1 {
            // Rapid happiness drops cause trust crashes
            happiness_change * 2.0 * (1.0 + self.radicalization)
        } else {
            happiness_change * 0.3
        };
        
        let economy_impact = if economy_change < -0.05 {
            // Economic crises cause disproportionate trust loss
            economy_change * 3.0
        } else {
            economy_change * 0.2
        };
        
        // Low trust creates positive feedback (more distrust)
        let distrust_feedback = if self.trust_in_government < 0.2 {
            -0.01 * (1.0 - self.trust_in_government)
        } else {
            0.0
        };
        
        let trust_change = happiness_impact + economy_impact + distrust_feedback;
        
        self.trust_in_government += trust_change;
        self.trust_in_government = self.trust_in_government.clamp(0.0, 1.0);
    }
}

use crate::engine::Economy;
