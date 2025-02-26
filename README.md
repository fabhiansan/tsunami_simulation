# Tsunami Simulation

A highly customizable Rust library for simulating tsunami evacuation scenarios with multiple agent types and behaviors, designed for agent-based modeling.

## Features

- Multi-agent simulation with configurable agent types (Child, Teen, Adult, Elder, and Custom)
- Realistic tsunami wave propagation with configurable parameters
- Multiple path-finding algorithms (Dijkstra, BFS, A*) for evacuation routes
- Shelter occupancy tracking with configurable capacity
- GeoJSON export for visualization and analysis
- Comprehensive configuration system for all simulation aspects
- Support for custom terrain types with variable traversal costs
- Diagonal or cardinal-only movement options
- Performance optimized with parallel computation where possible

## Installation

Add this to your `Cargo.toml`:
```toml
[dependencies]
tsunami_simulation = "0.1.1"
```

## Usage

### Basic Example

```rust
use tsunami_simulation::Simulation;
use std::path::Path;

fn main() -> std::io::Result<()> {
    // Initialize simulation with default configuration
    let mut simulation = Simulation::new(
        "data/grids/default_grid.asc",
        "data/population/default_population.asc"
    )?;
    
    // Run simulation steps
    while simulation.step() {
        println!(
            "Step: {} Tsunami Index: {}",
            simulation.current_step,
            simulation.tsunami_index
        );
    }
    
    // Export results
    std::fs::create_dir_all("output")?;
    tsunami_simulation::export_agents_to_geojson(
        &simulation.agent_data_collector,
        "output/step.geojson"
    )?;
    
    Ok(())
}
```

### Customized Example

```rust
use tsunami_simulation::*;
use tsunami_simulation::grid::GridConfig;
use tsunami_simulation::agent::AgentConfig;

fn main() -> std::io::Result<()> {
    // Create custom configurations
    let grid_config = GridConfig {
        blocked_penalty: 3,          // Higher penalty for blocked terrain
        allow_diagonal: true,        // Allow diagonal movement
        shelter_capacity: 100,       // Limit shelter capacity
        path_algorithm: "dijkstra".to_string(),
    };
    
    let agent_config = AgentConfig {
        base_speed: 3.0,             // Faster base speed
        speed_multipliers: [0.7, 0.9, 1.0, 0.6],  // Different speed multipliers
        type_weights: [10.0, 20.0, 50.0, 20.0],   // Different agent type distribution
    };
    
    let simulation_config = SimulationConfig {
        tsunami_delay: 20 * 60,      // Shorter delay before tsunami
        tsunami_speed_time: 20,      // Faster tsunami updates
        distribution_weights: [10, 20, 30, 20, 20],
        base_speed: agent_config.base_speed,
        agent_speed_multipliers: agent_config.speed_multipliers,
        agent_type_weights: agent_config.type_weights,
        data_collection_interval: 20,  // More frequent data collection
    };
    
    // Initialize simulation with custom configuration
    let mut simulation = Simulation::with_config(
        "data/grids/custom_grid.asc", 
        "data/population/custom_population.asc",
        simulation_config
    )?;
    
    // Run simulation for a specific number of steps
    simulation.run(Some(500))?;
    
    // Export results
    std::fs::create_dir_all("output")?;
    export_agents_to_geojson(
        &simulation.agent_data_collector,
        "output/custom_simulation.geojson"
    )?;
    
    Ok(())
}
```

## Configuration Options

### Simulation Configuration
- `tsunami_delay`: Time steps before tsunami starts
- `tsunami_speed_time`: Time steps between tsunami updates
- `distribution_weights`: Population distribution weights
- `base_speed`: Base movement speed for agents
- `agent_speed_multipliers`: Speed multipliers for each agent type
- `agent_type_weights`: Distribution weights for agent types
- `data_collection_interval`: Steps between data collection points

### Grid Configuration
- `blocked_penalty`: Movement cost penalty for blocked terrain
- `allow_diagonal`: Whether diagonal movement is allowed
- `shelter_capacity`: Maximum capacity of shelters (-1 for unlimited)
- `path_algorithm`: Path planning algorithm ("dijkstra", "bfs", "a_star")

### Agent Configuration
- `base_speed`: Base movement speed
- `speed_multipliers`: Speed multipliers for different agent types
- `type_weights`: Distribution weights for generating random agent types

## Data Format

The simulation requires two ASCII grid files:
- Grid file: Defines terrain, roads, and shelter locations
- Population file: Defines initial agent distribution

### Grid File Encoding
- `0`: Blocked terrain
- `1`: Road
- `20XX`: Shelter with ID XX
- `cX.X`: Custom terrain with X.X movement cost multiplier

## Output

The simulation generates:
- GeoJSON files with agent movements
- Shelter occupancy data
- Agent statistics
- Death counts by agent type

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Author

Fabhianto Maoludyo (fabhianto.maoludyo@gmail.com)

## Repository

[https://github.com/fabhiansan/tsunami_simulation](https://github.com/fabhiansan/tsunami_simulation)