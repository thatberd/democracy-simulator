use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economy {
    pub gdp: f32,
    pub unemployment: f32, // 0.0–1.0
    pub inequality: f32,   // 0.0–1.0
    pub previous_gdp: f32, // For lag effects
    pub previous_unemployment: f32,
    pub previous_inequality: f32,
    pub growth_trend: f32, // Positive = growing, negative = shrinking
}

impl Economy {
    pub fn new(gdp: f32, unemployment: f32, inequality: f32) -> Self {
        let unemployment = unemployment.clamp(0.0, 1.0);
        let inequality = inequality.clamp(0.0, 1.0);
        
        Self {
            gdp,
            unemployment,
            inequality,
            previous_gdp: gdp,
            previous_unemployment: unemployment,
            previous_inequality: inequality,
            growth_trend: 0.0,
        }
    }

    pub fn update(&mut self, government_ideology: f32, random_drift: f32) {
        // Store previous values for lag effects
        self.previous_gdp = self.gdp;
        self.previous_unemployment = self.unemployment;
        self.previous_inequality = self.inequality;
        
        // Calculate growth trend for momentum effects
        let gdp_change = self.gdp - self.previous_gdp;
        self.growth_trend = self.growth_trend * 0.8 + gdp_change * 0.2; // Exponential smoothing
        
        // Nonlinear ideology effects with diminishing returns
        let ideology_effect_gdp = government_ideology * 0.01 * (1.0 - self.inequality * 0.5); // Inequality reduces growth
        let ideology_effect_unemployment = -government_ideology.abs().powi(2) * 0.008; // Quadratic extremism penalty
        let ideology_effect_inequality = government_ideology * 0.008 * self.gdp; // GDP amplifies inequality changes
        
        // Lag effects: previous trends influence current changes
        let lag_gdp = self.growth_trend * 0.3;
        let lag_unemployment = (self.previous_unemployment - self.unemployment) * 0.2;
        
        // Threshold effects: crises amplify changes
        let crisis_multiplier = if self.unemployment > 0.4 || self.inequality > 0.7 {
            2.0 // Crisis mode: changes amplified
        } else {
            1.0
        };
        
        // Apply changes with nonlinear saturation
        let gdp_change = (ideology_effect_gdp + lag_gdp + random_drift * 0.02) * crisis_multiplier;
        let unemployment_change = (ideology_effect_unemployment + lag_unemployment + random_drift * 0.01) * crisis_multiplier;
        let inequality_change = (ideology_effect_inequality + random_drift * 0.015) * crisis_multiplier;
        
        // Apply with saturation using tanh
        self.gdp += gdp_change.tanh() * 0.05;
        self.unemployment += unemployment_change.tanh() * 0.03;
        self.inequality += inequality_change.tanh() * 0.02;
        
        // Clamp values
        self.gdp = self.gdp.max(0.1);
        self.unemployment = self.unemployment.clamp(0.0, 1.0);
        self.inequality = self.inequality.clamp(0.0, 1.0);
    }

    pub fn trigger_crisis(&mut self) {
        self.previous_gdp = self.gdp;
        self.previous_unemployment = self.unemployment;
        self.previous_inequality = self.inequality;
        
        self.gdp *= 0.8;
        self.unemployment = (self.unemployment * 1.5).min(1.0);
        self.inequality = (self.inequality * 1.2).min(1.0);
        self.growth_trend = -0.1; // Negative trend after crisis
    }

    pub fn trigger_boom(&mut self) {
        self.previous_gdp = self.gdp;
        self.previous_unemployment = self.unemployment;
        self.previous_inequality = self.inequality;
        
        self.gdp *= 1.3;
        self.unemployment *= 0.7;
        self.inequality *= 0.9;
        self.growth_trend = 0.1; // Positive trend after boom
    }
}
