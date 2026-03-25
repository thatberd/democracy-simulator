use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economy {
    pub gdp: f32,
    pub unemployment: f32, // 0.0–1.0
    pub inequality: f32,   // 0.0–1.0
}

impl Economy {
    pub fn new(gdp: f32, unemployment: f32, inequality: f32) -> Self {
        Self {
            gdp,
            unemployment: unemployment.clamp(0.0, 1.0),
            inequality: inequality.clamp(0.0, 1.0),
        }
    }

    pub fn update(&mut self, government_ideology: f32, random_drift: f32) {
        // Economic drift influenced by government ideology
        // Left-leaning governments tend to reduce inequality but may impact GDP
        // Right-leaning governments may increase GDP but also inequality
        
        let ideology_effect_gdp = government_ideology * 0.01; // Right-leaning boosts GDP
        let ideology_effect_unemployment = -government_ideology.abs() * 0.005; // Extremism increases unemployment
        let ideology_effect_inequality = government_ideology * 0.008; // Right-leaning increases inequality
        
        // Apply changes with random drift
        self.gdp += ideology_effect_gdp + random_drift * 0.02;
        self.unemployment += ideology_effect_unemployment + random_drift * 0.01;
        self.inequality += ideology_effect_inequality + random_drift * 0.015;
        
        // Clamp values
        self.gdp = self.gdp.max(0.1);
        self.unemployment = self.unemployment.clamp(0.0, 1.0);
        self.inequality = self.inequality.clamp(0.0, 1.0);
    }

    pub fn trigger_crisis(&mut self) {
        self.gdp *= 0.8;
        self.unemployment = (self.unemployment * 1.5).min(1.0);
        self.inequality = (self.inequality * 1.2).min(1.0);
    }

    pub fn trigger_boom(&mut self) {
        self.gdp *= 1.3;
        self.unemployment *= 0.7;
        self.inequality *= 0.9;
    }
}
