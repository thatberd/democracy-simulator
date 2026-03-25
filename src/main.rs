mod engine;
mod ui;

use std::env;
use ui::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let seed = if args.len() > 1 {
        args[1].parse::<u64>()
            .map_err(|_| "Invalid seed. Please provide a valid u64 number.")?
    } else {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen()
    };

    println!("Starting Democracy Simulator with seed: {}", seed);
    println!("The same seed will always produce identical results.");
    println!();
    
    let mut app = App::new(seed)?;
    app.run()?;

    Ok(())
}
