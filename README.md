# Tsunami Simulation

A Rust library for simulating tsunami evacuation scenarios with multiple agent types and behaviors.

## Features

- Multi-agent simulation with different agent types (Child, Teen, Adult, Elder)
- Realistic tsunami wave propagation
- Path-finding algorithms for evacuation routes
- Shelter occupancy tracking
- GeoJSON export for visualization
- Configurable simulation parameters

## Installation

Add this to your `Cargo.toml`:
[dependencies]
tsunami_simulation = "0.1.0"

## Usage

Basic example:
```rust
rust
use tsunami_simulation::Simulation;
use std::path::Path;
fn main() -> std::io::Result<()> {
// Initialize simulation with grid and population data
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
export_agents_to_geojson(
&simulation.agent_data_collector,
"output/step.geojson"
)?;
Ok(())
}
```


## Data Format

The simulation requires two ASCII grid files:
- Grid file: Defines terrain, roads, and shelter locations
- Population file: Defines initial agent distribution

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