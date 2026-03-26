# Democracy Simulator

A terminal-based autonomous democracy simulator built in Rust. The simulation runs entirely autonomously after initialization, with deterministic behavior driven by a numeric seed.

## Features

- **Deterministic Simulation**: Same seed always produces identical results
- **Autonomous Operation**: No player input required after initialization
- **Real-time TUI**: Terminal-based interface with live statistics
- **Complex Adaptive System**: Emergent polarization, clustering, tipping points, and social unrest
- **Local Interactions**: Citizens influence neighbors, not global averages
- **Nonlinear Dynamics**: Threshold effects, feedback loops, and cascading failures
- **Recovery Dynamics**: Societal cycles with collapse and reform movements
- **Event Cooldowns**: Realistic pacing prevents event spam
- **Memory-Based Behavior**: Citizens react to trends, not just current values
- **Echo Chambers**: Similar groups reinforce internally, opposing groups stop blending
- **Policy Lag**: Government decisions have delayed economic effects (10 ticks)
- **Natural Stabilization**: Very slow recovery prevents permanent deadlock

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

Each citizen has seven core attributes:
- **Ideology**: -1.0 (far left) to 1.0 (far right)
- **Happiness**: 0.0 to 1.0
- **Trust in Government**: 0.0 to 1.0
- **Radicalization**: 0.0 to 1.0 (how extreme/committed to ideology)
- **Previous Values**: Track changes for lag effects and dynamics
- **Memory Fields**: Past happiness and trust for trend-based behavior
- **Natural Drift**: Very slow recovery to prevent permanent extreme states

### Economy

- **GDP**: Economic output with growth trend tracking
- **Unemployment**: 0.0 to 1.0 (0% to 100%)
- **Inequality**: 0.0 to 1.0 (0% to 100%)
- **Previous Values**: Lag effects for momentum and cascading changes
- **Growth Trend**: Economic momentum (positive/negative)

### Government

- **Ideology**: Current ruling ideology
- **Term Remaining**: Ticks until next election
- **Policy Queue**: 15-tick history for delayed economic effects
- **Inertia**: 0.8 factor smooths ideological transitions between elections

## Simulation Dynamics

### Tick Cycle

Every simulation tick (100ms real-time):

1. **Local Citizen Interactions**:
   - Each citizen samples 3-8 random neighbors
   - Ideology influenced by local average, not global
   - **Ideological Repulsion**: Citizens >0.5 apart push further away, creating polarization
   - **Trust-Based Instability**: Low trust (<0.2) causes chaotic ideological shifts
   - **Echo Chambers**: Similar groups (similarity >0.7) get 2x reinforcement, opposing groups (similarity <0.3) get 0.2x influence
   - 50% of population engages in pairwise interactions

2. **Social Dynamics**:
   - Happiness affected by economy with threshold effects
   - Trust changes with memory-based trend amplification
   - Radicalization increases with extremeness and low trust
   - **Memory Updates**: Citizens store past values for trend-based behavior
   - **Natural Stabilization**: Very slow drift (+0.001) prevents permanent deadlock
   - **Inequality-Driven Polarization**: High inequality (>0.7) pushes citizens toward extremes

3. **Economic Updates**:
   - **Policy Lag**: Economy responds to government ideology from 10 ticks ago
   - Lag effects with previous values tracking
   - Growth trend momentum
   - Crisis amplification multipliers
   - Nonlinear saturation using sigmoid functions

4. **Government Updates**:
   - **Inertia**: 0.8 factor smooths ideological transitions
   - Policy queue updates for lagged effects
   - Term countdown

5. **Social Events** (with cooldowns):
   - **Protests**: Trust <0.2 AND happiness <0.3, with fatigue system (20-tick cooldown)
   - **Reform Movements**: Low trust + high radicalization/inequality triggers recovery (40-tick cooldown)
   - **Economic Crises**: Random chance with contextual triggers (30-tick cooldown)
   - **Social Harmony**: High trust + low radicalization (60-tick cooldown)
   - **Narrative Logging**: Events include contextual reasons and state snapshots

### Emergent Behaviors

The simulation produces complex emergent behaviors from simple rules:

- **Political Polarization**: Ideological repulsion creates distinct clusters and echo chambers
- **Faction Formation**: Citizens naturally group by ideology proximity through repulsion dynamics
- **Tipping Points**: Threshold effects cause sudden societal shifts and regime instability
- **Social Unrest**: Low trust amplifies chaos, creating unpredictable ideological swings
- **Economic Cascades**: Lag effects amplify booms and busts
- **Unexpected Elections**: Turnout and noise create surprising outcomes
- **Radicalization Cycles**: Feedback loops between trust and extremism
- **Societal Cycles**: Stability → decline → crisis → reform → recovery → repeat
- **Reform Movements**: System can recover from collapse through citizen-driven reforms
- **Protest Waves**: Fatigue system creates realistic protest patterns, not constant spam
- **Memory-Driven Behavior**: Citizens react to trends, creating delayed reactions and momentum
- **Policy Delay Effects**: Government decisions create economic cycles with 10-tick lag
- **Natural Recovery**: Very slow stabilization prevents permanent extreme states

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

## Recovery Dynamics

The simulation includes sophisticated recovery mechanisms that prevent permanent collapse:

### Reform Movements

- **Triggers**: Low trust (<0.2) + high radicalization (>0.6) OR high inequality (>0.6)
- **Effects**: +0.2 trust, +0.1 happiness, ×0.8 radicalization, reduced inequality
- **Cooldown**: 40 ticks to prevent reform spam
- **Narrative**: Contextual logging explains reform causes

### Protest Fatigue System

- **History Tracking**: Last 20 ticks of protest activity
- **Fatigue Multipliers**: 0.3 (high), 0.6 (moderate), 1.0 (none)
- **Wave Patterns**: Creates realistic protest cycles instead of constant spam
- **Cooldown**: 20 ticks between protest events

### Natural Stabilization

- **Very Slow Recovery**: +0.001 trust/happiness per tick when <0.1
- **Extreme State Dampening**: ×0.995 for ideologies >0.8
- **Prevents Deadlock**: System can always recover from extreme states
- **Long-Term Cycles**: Creates believable societal recovery patterns

### Event Cooldowns

- **Economic Crises**: 30 ticks
- **Protests**: 20 ticks
- **Reform Movements**: 40 ticks
- **Social Harmony**: 60 ticks
- **Purpose**: Realistic pacing and cleaner event logs

### Societal Cycles

The system now produces complete cycles:
1. **Stability** → High trust, functioning institutions
2. **Decline** → Rising inequality, falling trust
3. **Crisis** → Protests, economic instability
4. **Reform** → Recovery movements emerge
5. **Recovery** → Trust restored, system stabilizes
6. **Repeat** → New cycles begin with different characteristics

### Expected Societal Outcomes

Different seeds produce varying societal patterns:

- **Stable Democracies**: Balanced trust, moderate polarization, occasional reforms
- **Polarized Societies**: High faction clustering, election volatility, echo chamber effects
- **Chaotic Collapse**: Low trust, high radicalization, frequent protests followed by reforms
- **Economic Boom/Bust Cycles**: Policy lag creates momentum and crashes with delayed effects
- **Recovery Cycles**: Societies that collapse can recover through reform movements
- **Protest Waves**: Periods of unrest followed by fatigue and stabilization
- **Long-Term Stability**: Natural drift prevents permanent extreme states over time

## License

This project is open source. Feel free to modify and experiment with different simulation parameters!
