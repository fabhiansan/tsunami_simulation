use tsunami_simulation::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let data_dir = Path::new("data");
    let grid_path = data_dir.join("grids/default_grid.asc");
    let population_path = data_dir.join("population/default_population.asc");

    let mut simulation = Simulation::new(
        grid_path.to_str().unwrap(),
        population_path.to_str().unwrap()
    )?;

    while simulation.step() {
        println!(
            "Step: {} Tsunami Index: {}",
            simulation.current_step,
            simulation.tsunami_index
        );
    }

    std::fs::create_dir_all("output")?;
    export_agents_to_geojson(
        &simulation.agent_data_collector,
        "output/step.geojson"
    )?;

    Ok(())
} 