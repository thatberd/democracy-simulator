# Democracy Simulator

<p align="center"><img src="assets/img/logo.svg" width="100" height="100"></p>

A terminal-based autonomous democracy simulation built in Rust. The simulation runs entirely autonomously after initialization, with deterministic behavior driven by a numeric seed.

<img src="assets/img/demo.gif"></p>

## Features

- **Deterministic Simulation**: Same seed always produces identical results
- **Autonomous Operation**: No player input required after initialization
- **Modern Terminal UI**: Beautiful rounded corners and intuitive interface
- **Mouse Support**: Full mouse interaction with scrolling and clicking
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
- **Dynamic Instability System**: Bad conditions create pressure for change
- **Anti-Equilibrium Mechanisms**: Multiple systems prevent permanent static states
- **Persistent Reform Effects**: 30-tick reform duration with gradual decay
- **Instability Pressure**: Event probabilities scale with system health (1.0-2.0x)
- **Save/Load System**: Save and restore complete simulation state at any time
- **Configuration Validation**: Prevents invalid parameter combinations
- **Comprehensive Testing**: Extensive test suite with property-based validation
- **Enhanced Documentation**: Detailed algorithm explanations and performance notes

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

# Run with interactive configuration
cargo run --release --interactive

# Run with preset configuration
cargo run --release --preset collapse --seed 12345

# Run with custom parameters
cargo run --release --citizens 1500 --inequality 0.8 --trust 0.2 --volatility 0.7

# Save simulation state
cargo run --release --save my_simulation.json --seed 12345

# Load and continue saved simulation
cargo run --release --load my_simulation.json

# Test new presets
cargo run --release --preset utopia --seed 42
cargo run --release --preset revolution --seed 999
```

## Usage

### Command Line Interface

The democracy simulator now supports a rich CLI interface with multiple configuration options:

```bash
cargo run -- [OPTIONS]
```

#### Options

- `--seed <SEED>`: Random seed for deterministic simulation
- `--citizens <CITIZENS>`: Number of citizens (100-5000)
- `--inequality <INEQUALITY>`: Initial inequality (0.0-1.0)
- `--trust <TRUST>`: Initial trust (0.0-1.0)
- `--volatility <VOLATILITY>`: Economic volatility (0.0-1.0)
- `--preset <PRESET>`: Preset configuration (`collapse`, `stable`, `polarized`, `utopia`, `dystopia`, `revolution`)
- `--interactive`: Launch interactive TUI configuration mode
- `--save <PATH>`: Save simulation state to file on exit
- `--load <PATH>`: Load simulation state from file

#### Presets

- **collapse**: High inequality (0.8), low trust (0.1), high volatility (0.7), 1500 citizens
- **stable**: Balanced inequality (0.3), high trust (0.7), low volatility (0.2), 1200 citizens
- **polarized**: High population (2000), moderate inequality (0.6), moderate trust (0.4), medium volatility (0.5)
- **utopia**: Low inequality (0.1), high trust (0.9), low volatility (0.1), 800 citizens
- **dystopia**: Extreme inequality (0.9), very low trust (0.05), high volatility (0.8), 3000 citizens
- **revolution**: High inequality (0.75), low trust (0.15), high volatility (0.6), 1800 citizens

#### Priority Logic

1. `--interactive`: Launch TUI setup screen (overrides other options)
2. `--preset`: Load preset configuration
3. Individual parameters: Override preset values
4. Defaults: Used if no options provided

### Interactive Configuration Mode

Launch with `--interactive` to configure parameters in a TUI before simulation starts:

**Controls:**
- **↑/↓**: Select field
- **Enter**: Edit selected field
- **Tab**: Next field
- **Backspace**: Delete character
- **s**: Start simulation
- **q**: Quit

**Fields:**
- Citizens (100-5000)
- Inequality (0.0-1.0)
- Trust (0.0-1.0)
- Volatility (0.0-1.0)

### Deterministic Seed Generation

The simulator uses config-driven seed generation:

- Configuration parameters are hashed to create deterministic seeds
- Same configuration always produces the same seed
- Manual `--seed` overrides automatic generation
- Ensures reproducible scenarios across different systems

#### Examples

```bash
# Interactive mode with preset starting point
cargo run -- --interactive --preset stable

# Custom scenario with deterministic seed
cargo run -- --citizens 2000 --inequality 0.6 --trust 0.3 --seed 42

# Reproduce collapse scenario
cargo run -- --preset collapse --seed 99999
```

### Controls

#### Keyboard Controls

- **q**: Quit the simulator
- **p**: Pause/Resume simulation
- **r**: Reset with new random seed

#### Event Log Navigation

- **↑↓**: Scroll through events
- **PgUp/PgDn**: Page up/down through events
- **Home/End**: Jump to top/bottom of event log

#### Mouse Controls

- **Scroll Wheel**: Navigate through event log
- **Left Click**: 
  - Click in event area to jump to that position
  - Click in controls area to pause/resume
- **Right Click**: Quit the simulator
- **Middle Click**: Reset simulation and scroll to top

#### Visual Features

- **Rounded Corners**: All UI blocks use rounded borders for a modern appearance
- **Color-coded Information**: Different colors for different data types
- **Real-time Updates**: Smooth 10 FPS updates with live statistics

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
   - **Trust-Based Instability**: Low trust (<0.2) causes chaotic ideological shifts with 1.5x polarization factor
   - **Happiness-Driven Drift**: Unhappy citizens (happiness < 0.2) experience ideological movement (30% chance per tick)
   - **Echo Chambers**: Similar groups (similarity >0.7) get 2x reinforcement, opposing groups (similarity <0.3) get 0.2x influence
   - **Weak Long-Term Evolution**: 10% chance per tick of ±0.01 ideological drift to prevent permanent center lock
   - 50% of population engages in pairwise interactions

2. **Social Dynamics**:
   - Happiness affected by economy with stronger feedback loops
   - Trust changes with memory-based trend amplification
   - Radicalization increases with extremeness and low trust
   - **Memory Updates**: Citizens store past values for trend-based behavior
   - **Natural Stabilization**: Very slow drift (+0.001) prevents permanent deadlock
   - **Inequality-Driven Polarization**: High inequality (>0.7) pushes citizens toward extremes
   - **Economic-Happiness Coupling**: Stronger unemployment (-0.8), inequality (-0.5), and GDP (0.3) impacts

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

5. **Social Events** (with cooldowns and instability scaling):
   - **Protests**: Trust <0.2 AND happiness <0.3, with fatigue system (20-tick cooldown)
   - **Latent Unrest Trigger**: Catastrophic conditions (happiness <0.1 AND trust <0.1) force uprisings (5% chance)
   - **Reform Movements**: Low trust + high radicalization/inequality triggers recovery (40-tick cooldown)
   - **Persistent Reform System**: 30-tick duration with 2x strength multiplier and ongoing effects
   - **Economic Crises**: Random chance with contextual triggers (30-tick cooldown)
   - **Social Harmony**: High trust + low radicalization (60-tick cooldown)
   - **Instability Pressure**: Event probabilities scale 1.0-2.0x based on system health
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
- **Pressure-Driven Change**: Low happiness/trust actively drives polarization and unrest
- **Dynamic Event Probability**: Unhealthy systems become more reactive and event-prone
- **Economic-Happiness Coupling**: Stronger feedback loops prevent passive stability
- **Anti-Equilibrium Dynamics**: Multiple mechanisms prevent permanent static states

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
├── config.rs         # Configuration structs and CLI interface
├── engine/           # Simulation engine (deterministic, no UI)
│   ├── mod.rs
│   ├── citizen.rs    # Citizen model and behavior
│   ├── economy.rs    # Economic model
│   ├── government.rs # Government and elections
│   ├── state.rs      # World state and RNG management
│   ├── simulation.rs # Main simulation loop
│   └── tests.rs      # Comprehensive test suite
└── ui/               # Terminal user interface
    ├── mod.rs
    ├── app.rs        # Main application and event handling
    ├── config_screen.rs # Interactive TUI configuration
    └── renderer.rs   # TUI rendering logic
```

## Testing

The democracy simulator includes a comprehensive test suite to ensure reliability and correctness:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test modules
cargo test citizen_tests
cargo test economy_tests
cargo test integration_tests
```

### Test Coverage

- **Unit Tests**: Individual component testing (citizens, economy, government, state)
- **Integration Tests**: Full simulation workflow testing
- **Property-Based Tests**: Edge case validation using `proptest`
- **Determinism Tests**: Verify same seeds produce identical results
- **Serialization Tests**: Save/load functionality validation
- **Long-Running Tests**: Stability over thousands of ticks

### Configuration Validation

The simulator validates all configurations before starting:

```bash
# This will show validation errors
cargo run -- --citizens 50  # Too few citizens
cargo run -- --inequality 1.5  # Invalid range
```

Validation checks include:
- Citizen count bounds (100-5000)
- Parameter ranges (0.0-1.0 for scaled values)
- Logical consistency between parameters
- Prevention of unrealistic scenario combinations

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

## Anti-Equilibrium Systems

The simulation includes comprehensive mechanisms to prevent permanent static states and ensure that bad conditions create pressure for change:

### Pressure-Driven Instability

- **Happiness-Driven Ideological Drift**: Citizens with happiness < 0.2 experience ideological movement (30% chance per tick), breaking static center equilibrium
- **Trust-Amplified Polarization**: Low trust (<0.2) increases ideological divergence with 1.5x multiplier, preventing stable stagnation
- **Economic-Happiness Coupling**: Stronger feedback loops (unemployment -0.8, inequality -0.5, GDP 0.3) prevent passive stability

### Forced Unrest Mechanisms

- **Latent Unrest Trigger**: Catastrophic conditions (happiness < 0.1 AND trust < 0.1) force uprisings with 5% chance per tick
- **Instability Pressure**: Event probabilities scale 1.0-2.0x based on system health: `instability = (1.0 - trust) + (1.0 - happiness)`
- **Weak Long-Term Evolution**: 10% chance per tick of ±0.01 ideological drift prevents permanent center lock

### Dynamic Reform Recovery

- **Persistent Reform System**: 30-tick duration with 2x strength multiplier and gradual decay
- **Ongoing Reform Effects**: Continuous trust/happiness recovery and deradicalization during active reforms
- **Reform State Tracking**: Active reforms with countdown timers and strength decay

### Anti-Deadlock Guarantees

- **Multiple Pressure Points**: No single mechanism can be bypassed - bad conditions always create instability
- **Threshold Cascades**: Multiple systems trigger when conditions deteriorate sufficiently
- **Evolutionary Pressure**: Even stable systems experience weak long-term change

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
- **Pressure-Driven Evolution**: Bad conditions always create change through polarization, unrest, or reform
- **Dynamic Instability**: Unhealthy systems become increasingly reactive and event-prone
- **Anti-Equilibrium Dynamics**: No permanent static states - systems continuously evolve

## License

This project is open source. Feel free to modify and experiment with different simulation parameters!
