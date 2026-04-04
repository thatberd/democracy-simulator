use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::Path;
use clap::Parser;

#[derive(Clone, Debug, Hash)]
pub struct SimConfig {
    pub citizens: u32,
    pub initial_inequality: u32,   // scaled (0–1000)
    pub initial_trust: u32,        // scaled (0–1000)
    pub economic_volatility: u32,  // scaled (0–1000)
}

impl SimConfig {
    pub fn inequality_f32(&self) -> f32 {
        self.initial_inequality as f32 / 1000.0
    }

    pub fn trust_f32(&self) -> f32 {
        self.initial_trust as f32 / 1000.0
    }

    pub fn volatility_f32(&self) -> f32 {
        self.economic_volatility as f32 / 1000.0
    }

    /// Validates the configuration and returns a list of validation errors
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate citizen count
        if self.citizens < 100 {
            errors.push("Citizens must be at least 100".to_string());
        } else if self.citizens > 5000 {
            errors.push("Citizens cannot exceed 5000".to_string());
        }

        // Validate inequality (0-1000 scaled)
        if self.initial_inequality > 1000 {
            errors.push("Inequality cannot exceed 1.0".to_string());
        }

        // Validate trust (0-1000 scaled)
        if self.initial_trust > 1000 {
            errors.push("Trust cannot exceed 1.0".to_string());
        }

        // Validate volatility (0-1000 scaled)
        if self.economic_volatility > 1000 {
            errors.push("Volatility cannot exceed 1.0".to_string());
        }

        // Validate logical combinations
        if self.initial_inequality > 800 && self.initial_trust > 800 {
            errors.push("High inequality with high trust may create unrealistic scenarios".to_string());
        }

        if self.economic_volatility > 900 && self.initial_trust < 200 {
            errors.push("High volatility with low trust may cause excessive instability".to_string());
        }

        errors
    }

    /// Returns true if the configuration is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

pub fn config_to_seed(config: &SimConfig) -> u64 {
    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

#[derive(Parser)]
pub struct Cli {
    #[arg(long, help = "Random seed for deterministic simulation")]
    pub seed: Option<u64>,

    #[arg(long, help = "Number of citizens")]
    pub citizens: Option<u32>,

    #[arg(long, help = "Initial inequality (0.0-1.0)")]
    pub inequality: Option<f32>,

    #[arg(long, help = "Initial trust (0.0-1.0)")]
    pub trust: Option<f32>,

    #[arg(long, help = "Economic volatility (0.0-1.0)")]
    pub volatility: Option<f32>,

    #[arg(long, help = "Preset configuration (collapse, stable)")]
    pub preset: Option<String>,

    #[arg(long, help = "Interactive configuration mode")]
    pub interactive: bool,

    #[arg(long, help = "Save simulation state to file")]
    pub save: Option<String>,

    #[arg(long, help = "Load simulation state from file")]
    pub load: Option<String>,
}

pub fn default_config() -> SimConfig {
    SimConfig {
        citizens: 1000,
        initial_inequality: 500,
        initial_trust: 500,
        economic_volatility: 500,
    }
}

pub fn preset_config(name: &str) -> SimConfig {
    match name {
        "collapse" => SimConfig {
            citizens: 1500,
            initial_inequality: 800,
            initial_trust: 100,
            economic_volatility: 700,
        },
        "stable" => SimConfig {
            citizens: 1200,
            initial_inequality: 300,
            initial_trust: 700,
            economic_volatility: 200,
        },
        "polarized" => SimConfig {
            citizens: 2000,
            initial_inequality: 600,
            initial_trust: 400,
            economic_volatility: 500,
        },
        "utopia" => SimConfig {
            citizens: 800,
            initial_inequality: 100,
            initial_trust: 900,
            economic_volatility: 100,
        },
        "dystopia" => SimConfig {
            citizens: 3000,
            initial_inequality: 900,
            initial_trust: 50,
            economic_volatility: 800,
        },
        "revolution" => SimConfig {
            citizens: 1800,
            initial_inequality: 750,
            initial_trust: 150,
            economic_volatility: 600,
        },
        _ => default_config(),
    }
}

pub fn resolve_config(cli: &Cli) -> SimConfig {
    let mut config = if let Some(preset_name) = &cli.preset {
        preset_config(preset_name)
    } else {
        default_config()
    };

    // Override with individual CLI args
    if let Some(citizens) = cli.citizens {
        config.citizens = citizens;
    }
    if let Some(inequality) = cli.inequality {
        config.initial_inequality = (inequality.clamp(0.0, 1.0) * 1000.0) as u32;
    }
    if let Some(trust) = cli.trust {
        config.initial_trust = (trust.clamp(0.0, 1.0) * 1000.0) as u32;
    }
    if let Some(volatility) = cli.volatility {
        config.economic_volatility = (volatility.clamp(0.0, 1.0) * 1000.0) as u32;
    }

    config
}
