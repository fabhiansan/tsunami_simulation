use tsunami_simulation::*;
use tsunami_simulation::grid::{GridConfig, Terrain};
use tsunami_simulation::agent::{AgentConfig, AgentType};

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
    
    // Create grid with custom configuration
    let (mut grid, mut agents) = grid::load_grid_from_ascii_with_config(
        "./data_pacitan/agent2mboundariesjalan_reprojected.asc",
        grid_config
    )?;
    
    // Add a custom terrain type to demonstrate flexibility
    grid.terrain[10][10] = Terrain::Custom(1.5); // Terrain with 1.5x movement cost
    
    // Add a custom agent type to demonstrate flexibility
    agents.push(agent::Agent::with_config(
        agents.len(),
        20,
        20,
        AgentType::Custom(1.2), // Custom agent with 1.2x speed multiplier
        true,
        &agent_config
    ));
    
    // Initialize simulation with custom configuration
    let mut simulation = Simulation::with_config(
        "./data_pacitan/agent2mboundariesjalan_reprojected.asc", 
        "./data_pacitan/jalandantes2m.asc",
        simulation_config
    )?;
    
    // Run simulation for 500 steps or until completion
    println!("Running custom tsunami simulation...");
    simulation.run(Some(500))?;
    
    // Export results
    std::fs::create_dir_all("output")?;
    export_agents_to_geojson(
        &simulation.agent_data_collector,
        "output/custom_simulation.geojson"
    )?;
    
    println!("Simulation complete. Results saved to output/custom_simulation.geojson");
    Ok(())
}