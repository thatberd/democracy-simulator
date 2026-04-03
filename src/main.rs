mod engine;
mod ui;
mod config;

use clap::Parser;
use config::{Cli, resolve_config, config_to_seed};
use ui::App;
use ui::config_screen::ConfigScreen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

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

    let seed = cli.seed.unwrap_or_else(|| config_to_seed(&config));

    println!("Starting Democracy Simulator");
    println!("Config: {} citizens, inequality={:.3}, trust={:.3}, volatility={:.3}", 
             config.citizens, config.inequality_f32(), config.trust_f32(), config.volatility_f32());
    println!("Seed: {}", seed);
    println!("The same seed will always produce identical results.");
    println!();
    
    let mut app = App::new_with_config(seed, config)?;
    app.run()?;

    Ok(())
}
