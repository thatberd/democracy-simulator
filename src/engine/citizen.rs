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
    // Memory fields for trend-based behavior
    pub past_happiness: f32,  // happiness from N ticks ago
    pub past_trust: f32,      // trust from N ticks ago
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
            past_happiness: happiness, // Initialize with current values
            past_trust: trust,
        }
    }

    pub fn update_memory(&mut self) {
        // Shift memory: current becomes past, past becomes older
        self.past_happiness = self.previous_happiness;
        self.past_trust = self.previous_trust;
    }

    pub fn increase_polarization_from_inequality(&mut self, inequality: f32) {
        // High inequality increases polarization rate
        if inequality > 0.7 {
            let polarization_boost = (inequality - 0.7) * 0.02;
            // Push citizens toward more extreme positions
            if self.ideology > 0.0 {
                self.ideology = (self.ideology + polarization_boost).min(1.0);
            } else {
                self.ideology = (self.ideology - polarization_boost).max(-1.0);
            }
        }
    }

    pub fn apply_natural_stabilization_drift(&mut self, center_drift: f32) {
        // Very slow recovery to prevent permanent deadlock
        let trust_drift = 0.001;
        let happiness_drift = 0.001;
        let ideology_drift = 0.995; // Very slow pull toward center
        
        // Apply drift only when values are very low
        if self.trust_in_government < 0.1 {
            self.trust_in_government = (self.trust_in_government + trust_drift).min(0.15);
        }
        if self.happiness < 0.1 {
            self.happiness = (self.happiness + happiness_drift).min(0.15);
        }
        
        // Very slow ideological moderation over long time
        if self.ideology.abs() > 0.8 {
            self.ideology *= ideology_drift;
        }
        
        // PREVENT PERMANENT CENTER LOCK: Apply weak long-term drift
        self.ideology += center_drift;
        self.ideology = self.ideology.clamp(-1.0, 1.0);
    }

    /// Updates citizen ideology based on local social interactions and individual factors.
    /// 
    /// This is the core ideological dynamics function that combines multiple influences:
    /// 
    /// # Parameters
    /// - `local_avg_ideology`: Average ideology of sampled neighbors (-1.0 to 1.0)
    /// - `noise`: Random perturbation for individual variation
    /// - `chaos`: Additional instability factor when trust is low
    /// - `happiness_drift`: Ideological movement driven by unhappiness
    /// 
    /// # Algorithm Components:
    /// 
    /// ## 1. Trust-Based Social Influence
    /// Low trust increases susceptibility to peer influence (trust_factor = 1.0 - trust).
    /// This models how distrust in institutions drives people toward social groups.
    /// 
    /// ## 2. Happiness-Driven Susceptibility  
    /// Unhappy citizens (happiness < 0.3) are 50% more susceptible to influence.
    /// This models how dissatisfaction drives ideological change.
    /// 
    /// ## 3. Nonlinear Saturation
    /// Uses tanh() to saturate influence at extreme ideological differences.
    /// Prevents unrealistic jumps when neighbors are very different.
    /// 
    /// ## 4. Ideological Repulsion
    /// When distance > 0.5, citizens are pushed further away from local average.
    /// This creates polarization and prevents complete homogenization.
    /// 
    /// ## 5. Chaos Factor
    /// Additional randomness when trust < 0.2, scaled by hardship duration.
    /// Models unpredictable behavior in low-trust environments.
    /// 
    /// ## 6. Happiness-Driven Drift
    /// Unhappy citizens experience random ideological movement (30% chance per tick).
    /// Breaks static equilibrium and models how misery drives change.
    pub fn update_ideology_local(&mut self, local_avg_ideology: f32, noise: f32, chaos: f32, happiness_drift: f32) {
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
        
        self.ideology += nonlinear_diff * influence_strength + noise + repulsion_effect + chaos + happiness_drift;
        self.ideology = self.ideology.clamp(-1.0, 1.0);
        
        // Update radicalization based on extremeness and social conditions
        self.update_radicalization();
    }
    
    pub fn interact_with(&mut self, other: &Citizen) {
        // Citizen-to-citizen interaction with echo chamber effects
        let ideology_diff = other.ideology - self.ideology;
        let distance = ideology_diff.abs();
        
        // Calculate similarity for echo chamber effect
        let similarity = 1.0 - distance;
        
        // Echo chamber: similar groups reinforce internally, opposing groups stop blending
        let base_strength = 0.02;
        let echo_chamber_multiplier = if similarity > 0.7 {
            2.0 // Strong reinforcement within echo chambers
        } else if similarity < 0.3 {
            0.2 // Weak influence across opposing groups
        } else {
            1.0 // Normal influence for moderate similarity
        };
        
        // Distance-based decay still applies but is modified by echo chamber
        let distance_factor = (-distance * 3.0).exp();
        
        let influence = ideology_diff * base_strength * distance_factor * echo_chamber_multiplier;
        
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
        
        // Economic factors with stronger feedback effects
        let unemployment_impact = if economy.unemployment > 0.3 {
            // Threshold effect: high unemployment causes disproportionate unhappiness
            -(economy.unemployment - 0.3).powi(2) * 4.0 // Increased from 3.0
        } else {
            -economy.unemployment * 0.8 // Increased from 0.5
        };
        
        let inequality_impact = if economy.inequality > 0.6 {
            // High inequality triggers strong negative response
            -(economy.inequality - 0.6).powi(3) * 6.0 // Increased from 4.0
        } else {
            -economy.inequality * 0.5 // Increased from 0.3
        };
        
        // GDP changes affect happiness more strongly
        let gdp_trend = economy.gdp - economy.previous_gdp;
        let gdp_impact = if gdp_trend < -0.02 {
            // Falling GDP reduces happiness significantly
            gdp_trend * 3.0 // Increased from 2.0
        } else if gdp_trend > 0.02 {
            // Rising GDP improves happiness moderately
            gdp_trend * 1.2 // Increased from 0.8
        } else {
            0.0
        };
        
        // STRONGER ECONOMY → HAPPINESS COUPLING: direct economic effects
        let gdp_level_impact = (economy.gdp - 1.0) * 0.3; // Increased from 0.2
        let unemployment_direct = -economy.unemployment * 0.7; // Increased from 0.5
        
        // Political alignment with nonlinear amplification for extremists
        let ideology_diff = (self.ideology - government_ideology).abs();
        let alignment_factor = if ideology_diff > 0.5 {
            // Extremists experience stronger dissatisfaction
            -(ideology_diff * 0.6 * (1.0 + self.radicalization)) // Increased from 0.5
        } else {
            (1.0 - ideology_diff) * 0.3 // Increased from 0.2
        };
        
        // Trust creates feedback loop
        let trust_bonus = self.trust_in_government * 0.2; // Increased from 0.15
        
        // High inequality increases polarization effect on happiness
        let polarization_modifier = if economy.inequality > 0.7 {
            -self.radicalization * 0.3 // Increased from 0.2
        } else {
            0.0
        };
        
        // Inequality increases dissatisfaction and radicalization
        let inequality_radicalization = if economy.inequality > 0.8 {
            -economy.inequality * 0.2 * self.radicalization
        } else {
            0.0
        };
        
        let mut new_happiness = base_happiness + unemployment_impact + inequality_impact 
            + alignment_factor + trust_bonus + gdp_impact + polarization_modifier
            + gdp_level_impact + unemployment_direct + inequality_radicalization;
        
        // Apply sigmoid transformation for saturation effects
        new_happiness = (new_happiness * 4.0 - 2.0).tanh() * 0.5 + 0.5;
        
        self.happiness = new_happiness.clamp(0.0, 1.0);
    }

    pub fn update_trust(&mut self, happiness_change: f32, economy_change: f32) {
        // Store previous value
        self.previous_trust = self.trust_in_government;
        
        // Calculate trends for memory-based behavior
        let happiness_trend = self.happiness - self.past_happiness;
        let trust_trend = self.trust_in_government - self.past_trust;
        
        // Nonlinear response to changes with memory amplification
        let happiness_impact = if happiness_change < -0.1 {
            // Rapid happiness drops cause trust crashes
            let memory_multiplier = if happiness_trend < -0.05 { 1.5 } else { 1.0 }; // Amplify if declining trend
            happiness_change * 2.0 * (1.0 + self.radicalization) * memory_multiplier
        } else {
            // Slow trust recovery even when improving
            let recovery_damping = if happiness_trend > 0.05 { 0.5 } else { 1.0 }; // Dampen recovery
            happiness_change * 0.3 * recovery_damping
        };
        
        let economy_impact = if economy_change < -0.05 {
            // Economic crises cause disproportionate trust loss
            let memory_multiplier = if trust_trend < -0.02 { 1.3 } else { 1.0 }; // Amplify if trust was already falling
            economy_change * 3.0 * memory_multiplier
        } else {
            // Slow economic recovery trust gains
            let recovery_damping = if economy_change > 0.02 { 0.4 } else { 1.0 };
            economy_change * 0.2 * recovery_damping
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
