# Democracy Simulator

A terminal-based autonomous democracy simulator built in Rust. The simulation runs entirely autonomously after initialization, with deterministic behavior driven by a numeric seed.

## Features

- **Deterministic Simulation**: Same seed always produces identical results
- **Autonomous Operation**: No player input required after initialization
- **Real-time TUI**: Terminal-based interface with live statistics
- **Emergent Behavior**: Political polarization, economic cycles, and opinion shifts

## Technical Details

- **Language**: Rust
- **UI Framework**: ratatui + crossterm
- **Randomness**: Seeded RNG for reproducibility
- **Architecture**: Clean separation between simulation engine and UI

## Installation

### Prerequisites

- Rust 1.70+ installed
- Terminal that supports ANSI escape codes

### Build and Run

```bash
# Clone or download the project
cd democracy-simulator

# Build the project
cargo build --release

# Run with a specific seed
cargo run --release 12345

# Run with random seed
cargo run --release
```

## Usage

### Command Line Arguments

```bash
cargo run -- [SEED]
```

- `SEED`: Optional u64 number to seed the simulation. If omitted, a random seed is used.

### Controls

- **q**: Quit the simulator
- **p**: Pause/Resume simulation
- **r**: Reset with new random seed

## Simulation Model

### Citizens

Each citizen has three core attributes:
- **Ideology**: -1.0 (far left) to 1.0 (far right)
- **Happiness**: 0.0 to 1.0
- **Trust in Government**: 0.0 to 1.0

### Economy

- **GDP**: Economic output
- **Unemployment**: 0.0 to 1.0 (0% to 100%)
- **Inequality**: 0.0 to 1.0 (0% to 100%)

### Government

- **Ideology**: Current ruling ideology
- **Term Remaining**: Ticks until next election

## Simulation Dynamics

### Tick Cycle

Every simulation tick (100ms real-time):

1. **Citizen Updates**:
   - Ideology drifts toward global average with random noise
   - Happiness affected by economy and political alignment
   - Trust changes based on happiness and economic trends

2. **Economic Updates**:
   - Small random drift
   - Influenced by government ideology
   - Random events (crises, booms)

3. **Elections**:
   - Occur every 50 ticks
   - Citizens vote based on ideology proximity
   - Government ideology becomes median of citizen ideologies

### Emergent Behaviors

The simulation produces complex emergent behaviors:
- **Political Polarization**: Ideologies can cluster at extremes
- **Economic Cycles**: Boom and bust patterns
- **Stability vs Instability**: Periods of calm vs rapid change
- **Opinion Shifts**: Gradual or sudden changes in public sentiment

## Determinism

The simulation is fully deterministic:
- All randomness comes from a seeded `StdRng`
- No system time or external entropy sources
- Same seed → identical simulation state at every tick
- Serialization support for saving/loading states

## UI Layout

The terminal interface displays:

- **Header**: Current seed and tick counter
- **Statistics**: Citizen counts, averages, government info
- **Economy**: GDP, unemployment, inequality
- **Ideology Distribution**: ASCII bar chart
- **Event Log**: Major events (elections, crises, booms)
- **Controls**: Status and keyboard shortcuts

## Project Structure

```
src/
├── main.rs           # Entry point and CLI handling
├── engine/           # Simulation engine (deterministic, no UI)
│   ├── mod.rs
│   ├── citizen.rs    # Citizen model and behavior
│   ├── economy.rs    # Economic model
│   ├── government.rs # Government and elections
│   ├── state.rs      # World state and RNG management
│   └── simulation.rs # Main simulation loop
└── ui/               # Terminal user interface
    ├── mod.rs
    ├── app.rs        # Main application and event handling
    └── renderer.rs   # TUI rendering logic
```

## Testing Determinism

To verify the simulation is deterministic:

```bash
# Run with same seed multiple times
cargo run --release 42
# Observe identical behavior

# Or test programmatically
cargo test
```

## Performance

- Simulates 500-2000 citizens
- Runs at 10 FPS (100ms per tick)
- Minimal CPU usage
- Memory efficient (~few MB)

## License

This project is open source. Feel free to modify and experiment with different simulation parameters!
