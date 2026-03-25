# Democracy Simulator

A terminal-based autonomous democracy simulator built in Rust. The simulation runs entirely autonomously after initialization, with deterministic behavior driven by a numeric seed.

## Features

- **Deterministic Simulation**: Same seed always produces identical results
- **Autonomous Operation**: No player input required after initialization
- **Real-time TUI**: Terminal-based interface with live statistics
- **Complex Adaptive System**: Emergent polarization, clustering, tipping points, and social unrest
- **Local Interactions**: Citizens influence neighbors, not global averages
- **Nonlinear Dynamics**: Threshold effects, feedback loops, and cascading failures

## Technical Details

- **Language**: Rust
- **UI Framework**: ratatui + crossterm
- **Randomness**: Seeded RNG for reproducibility
- **Architecture**: Clean separation between simulation engine and UI
- **Complexity**: O(n) performance with ~1000 citizens efficiently

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

Each citizen has five core attributes:
- **Ideology**: -1.0 (far left) to 1.0 (far right)
- **Happiness**: 0.0 to 1.0
- **Trust in Government**: 0.0 to 1.0
- **Radicalization**: 0.0 to 1.0 (how extreme/committed to ideology)
- **Previous Values**: Track changes for lag effects and dynamics

### Economy

- **GDP**: Economic output with growth trend tracking
- **Unemployment**: 0.0 to 1.0 (0% to 100%)
- **Inequality**: 0.0 to 1.0 (0% to 100%)
- **Previous Values**: Lag effects for momentum and cascading changes
- **Growth Trend**: Economic momentum (positive/negative)

### Government

- **Ideology**: Current ruling ideology
- **Term Remaining**: Ticks until next election

## Simulation Dynamics

### Tick Cycle

Every simulation tick (100ms real-time):

1. **Local Citizen Interactions**:
   - Each citizen samples 3-8 random neighbors
   - Ideology influenced by local average, not global
   - **Ideological Repulsion**: Citizens >0.5 apart push further away, creating polarization
   - **Trust-Based Instability**: Low trust (<0.2) causes chaotic ideological shifts
   - 50% of population engages in lightweight pairwise interactions (0.01 influence)

2. **Social Dynamics**:
   - Happiness affected by economy with threshold effects
   - Trust changes with nonlinear amplification during crises
   - Radicalization increases with extremeness and low trust
   - Feedback loops between all metrics

3. **Economic Updates**:
   - Lag effects with previous values tracking
   - Growth trend momentum
   - Crisis amplification multipliers
   - Nonlinear saturation using sigmoid functions

4. **Imperfect Elections**:
   - 60-90% voter turnout simulation
   - Individual noise and extremist bias
   - Systemic noise for media influence
   - Occur every 50 ticks

5. **Social Events**:
   - Protests when trust < 0.3 AND radicalization > 0.5
   - Social harmony during positive conditions
   - Economic crises and booms with cascading effects

### Emergent Behaviors

The simulation produces complex emergent behaviors from simple rules:

- **Political Polarization**: Ideological repulsion creates distinct clusters and echo chambers
- **Faction Formation**: Citizens naturally group by ideology proximity through repulsion dynamics
- **Tipping Points**: Threshold effects cause sudden societal shifts and regime instability
- **Social Unrest**: Low trust amplifies chaos, creating unpredictable ideological swings
- **Economic Cascades**: Lag effects amplify booms and busts
- **Unexpected Elections**: Turnout and noise create surprising outcomes
- **Radicalization Cycles**: Feedback loops between trust and extremism
- **Stable vs Chaotic Societies**: Different seeds produce varying societal outcomes
- **Emergent Clustering**: Local interactions + repulsion become multi-peak ideology distributions (`█   █` vs ` █ `)

### Complex Adaptive System Properties

- **Self-Organization**: Citizens form clusters without central control
- **Nonlinearity**: Small changes can cause large effects
- **Emergence**: System-level patterns from local interactions
- **Adaptation**: Society responds to economic and political stress
- **Path Dependence**: History influences future through lag effects

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

- Simulates 500-2000 citizens efficiently
- Runs at 10 FPS (100ms per tick)
- O(n) complexity with local sampling (no O(n²) pairwise operations)
- Minimal CPU usage with optimized interactions
- Memory efficient (~few MB)
- Supports complex adaptive system dynamics at scale

## Complex Adaptive System Design

### Core Principles

The simulation follows complex adaptive system principles:

- **Simple Rules → Complex Outcomes**: Individual citizen interactions create societal-level patterns
- **Local Interactions**: Citizens influence neighbors, not global averages
- **Nonlinear Dynamics**: Threshold effects and feedback loops create tipping points
- **Deterministic Chaos**: Same seed produces identical but unpredictable-looking behavior
- **Emergent Factions**: No hardcoded factions - they emerge from ideology proximity

### Key Mechanisms

1. **Local Sampling**: Each citizen samples 3-8 neighbors per tick
2. **Distance-Based Influence**: Ideological proximity determines interaction strength
3. **Threshold Effects**: Crises amplify changes, low trust creates distrust cascades
4. **Lag Effects**: Previous values influence current changes (momentum)
5. **Imperfect Information**: Elections have noise and turnout variation

### Expected Societal Outcomes

Different seeds produce varying societal patterns:
- **Stable Democracies**: Balanced trust, moderate polarization
- **Polarized Societies**: High faction clustering, election volatility
- **Chaotic Collapse**: Low trust, high radicalization, frequent protests
- **Economic Boom/Bust Cycles**: Lag effects create momentum and crashes

## License

This project is open source. Feel free to modify and experiment with different simulation parameters!
