use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
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
