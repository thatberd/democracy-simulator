mod engine;
mod ui;
mod config;

use clap::Parser;
use config::{Cli, resolve_config, config_to_seed};
use ui::App;
use ui::config_screen::ConfigScreen;
use std::fs;
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle load command
    if let Some(load_path) = &cli.load {
        return load_and_run_simulation(load_path);
    }

    let config = if cli.interactive {
        // Interactive mode: start with default or preset config
        let initial_config = if let Some(preset_name) = &cli.preset {
            config::preset_config(preset_name)
        } else {
            config::default_config()
        };
        
        let mut config_screen = ConfigScreen::from_config(initial_config)?;
        config_screen.run()?
    } else {
        // CLI mode: resolve config from arguments and presets
        resolve_config(&cli)
    };

    // Validate configuration
    let validation_errors = config.validate();
    if !validation_errors.is_empty() {
        eprintln!("Configuration validation errors:");
        for error in validation_errors {
            eprintln!("  - {}", error);
        }
        eprintln!();
        eprintln!("Please fix these errors and try again.");
        std::process::exit(1);
    }

    let seed = cli.seed.unwrap_or_else(|| config_to_seed(&config));

    println!("Starting Democracy Simulator");
    println!("Config: {} citizens, inequality={:.3}, trust={:.3}, volatility={:.3}", 
             config.citizens, config.inequality_f32(), config.trust_f32(), config.volatility_f32());
    println!("Seed: {}", seed);
    println!("The same seed will always produce identical results.");
    println!();
    
    let mut app = App::new_with_config(seed, config)?;
    
    // Handle save command during setup
    if let Some(save_path) = &cli.save {
        app.set_auto_save_path(save_path.clone());
    }
    
    app.run()?;

    Ok(())
}

fn load_and_run_simulation(load_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading simulation from: {}", load_path);
    
    // Read the saved state
    let mut file = fs::File::open(load_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // Deserialize the state
    let state = engine::State::deserialize_state(&contents)?;
    
    println!("Loaded simulation:");
    println!("  Seed: {}", state.seed);
    println!("  Tick: {}", state.tick);
    println!("  Citizens: {}", state.citizens.len());
    println!("  Avg Ideology: {:.3}", state.get_average_ideology_immutable());
    println!("  Avg Happiness: {:.3}", state.get_average_happiness_immutable());
    println!("  Avg Trust: {:.3}", state.get_average_trust_immutable());
    println!();
    
    // Create app with loaded state
    let mut app = App::new_with_state(state)?;
    
    // Handle save command for loaded simulation
    let cli = Cli::parse();
    if let Some(save_path) = &cli.save {
        app.set_auto_save_path(save_path.clone());
        println!("Auto-save enabled: {}", save_path);
    }
    
    app.run()?;
    
    Ok(())
}
