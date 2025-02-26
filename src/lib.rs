mod game;

use game::agent::{Agent, AgentType};
use game::game::Model;
use game::grid::{load_grid_from_ascii, Grid, Terrain};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use rayon::prelude::*;

// Re-export important types and modules
pub use game::agent;
pub use game::game as simulation_game; // Renamed to avoid conflict
pub use game::grid;

/// Configuration for the tsunami simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Delay in time steps before tsunami starts (default: 30 * 60)
    pub tsunami_delay: u32,
    /// Time steps between tsunami propagation updates (default: 28)
    pub tsunami_speed_time: u32,
    /// Distribution weights for population distribution (default: [10, 20, 30, 15, 20])
    pub distribution_weights: [i32; 5],
    /// Base movement speed for agents in meters per second (default: 2.66)
    pub base_speed: f64,
    /// Speed multipliers for different agent types [Child, Teen, Adult, Elder]
    pub agent_speed_multipliers: [f64; 4],
    /// Distribution weights for agent types [Child, Teen, Adult, Elder]
    pub agent_type_weights: [f64; 4],
    /// Interval for collecting agent data (default: 30 steps)
    pub data_collection_interval: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            tsunami_delay: 30 * 60,
            tsunami_speed_time: 28,
            distribution_weights: [10, 20, 30, 15, 20],
            base_speed: 2.66,
            agent_speed_multipliers: [0.8, 1.0, 1.0, 0.7],
            agent_type_weights: [6.21, 13.41, 59.10, 19.89],
            data_collection_interval: 30,
        }
    }
}

// Legacy constants for backward compatibility
pub const TSUNAMI_DELAY: u32 = 30 * 60;
pub const TSUNAMI_SPEED_TIME: u32 = 28;
pub const DISTRIBUTION_WEIGHTS: [i32; 5] = [10, 20, 30, 15, 20];

#[derive(Serialize, Deserialize)]
pub struct ShelterAgentTypeData {
    pub child: u32,
    pub teen: u32,
    pub adult: u32,
    pub elder: u32,
}

impl Default for ShelterAgentTypeData {
    fn default() -> Self {
        ShelterAgentTypeData {
            child: 0,
            teen: 0,
            adult: 0,
            elder: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShelterData {
    pub step: u32,
    pub shelters: HashMap<String, ShelterAgentTypeData>,
    pub total_dead_agents: usize,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SimulationData {
    pub records: Vec<ShelterData>,
}

#[derive(Serialize, Deserialize)]
pub struct AgentStatistics {
    pub total_agents: usize,
    pub agent_types: HashMap<String, usize>,
}

#[derive(Clone)]
pub struct AgentDataCollector {
    data: Vec<AgentStepData>,
    grid: Grid,
}

#[derive(Clone)]
struct AgentStepData {
    x: f64,
    y: f64,
    id: usize,
    agent_type: String,
    is_on_road: bool,
    speed: u32,
    step: u32,
}

impl AgentDataCollector {
    pub fn new(grid: Grid) -> Self {
        Self {
            data: Vec::new(),
            grid,
        }
    }

    pub fn collect_step(&mut self, model: &Model, step: u32) {
        for agent in &model.agents {
            if agent.is_alive {
                let real_x = model.grid.xllcorner + (agent.x as f64 * model.grid.cellsize);
                let real_y = model.grid.yllcorner
                    + (-1.0 * agent.y as f64 * model.grid.cellsize)
                    + (model.grid.nrow as f64 * model.grid.cellsize);

                self.data.push(AgentStepData {
                    x: real_x,
                    y: real_y,
                    id: agent.id,
                    agent_type: format!("{:?}", agent.agent_type),
                    is_on_road: agent.is_on_road,
                    speed: agent.speed,
                    step,
                });
            }
        }
    }

    pub fn get_data(&self) -> &Vec<AgentStepData> {
        &self.data
    }
}

pub fn load_population_and_create_agents(
    path: &str,
    ncols: u32,
    nrows: u32,
    grid: &mut Grid,
    agents: &mut Vec<Agent>,
    next_agent_id: &mut usize,
) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Skip header lines
    for _ in 0..6 {
        lines.next();
    }

    let mut population: Vec<Vec<u32>> = Vec::with_capacity(nrows as usize);
    for line in lines {
        let line = line?;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < ncols as usize {
            continue;
        }
        let row: Vec<u32> = tokens
            .iter()
            .take(ncols as usize)
            .map(|token| token.parse::<u32>().unwrap_or(0))
            .collect();
        population.push(row);
    }

    if population.len() != nrows as usize {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Population data dimensions don't match grid",
        ));
    }

    grid.population = population.clone();

    for (y, row) in population.iter().enumerate() {
        for (x, &pop) in row.iter().enumerate() {
            if pop != 0 {
                let is_on_road = grid.terrain[y][x] == Terrain::Road;
                let agent_type = AgentType::random();

                let mut agent = Agent::new(
                    *next_agent_id,
                    x as u32,
                    y as u32,
                    agent_type,
                    is_on_road,
                );
                agent.remaining_steps = agent.speed;

                grid.add_agent(x as u32, y as u32, agent.id);
                agents.push(agent);
                *next_agent_id += 1;
            }
        }
    }

    Ok(())
}

pub fn export_agents_to_geojson(collector: &AgentDataCollector, filename: &str) -> io::Result<()> {
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;

    let mut grouped_data: HashMap<(u32, String), Vec<Vec<f64>>> = HashMap::new();

    for agent_data in collector.get_data() {
        let key = (agent_data.step, agent_data.agent_type.clone());
        let coordinates = grouped_data.entry(key).or_insert_with(Vec::new);
        coordinates.push(vec![agent_data.x, agent_data.y]);
    }

    let features: Vec<Value> = grouped_data
        .into_iter()
        .map(|((step, agent_type), coordinates)| {
            json!({
                "type": "Feature",
                "geometry": {
                    "type": "MultiPoint",
                    "coordinates": coordinates
                },
                "properties": {
                    "timestamp": step,
                    "agent_type": agent_type
                }
            })
        })
        .collect();

    let geojson = json!({
        "type": "FeatureCollection",
        "crs": {
            "type": "name",
            "properties": {
                "name": "EPSG:4326"
            }
        },
        "features": features
    });

    let mut file = File::create(filename)?;
    file.write_all(serde_json::to_string_pretty(&geojson)?.as_bytes())?;

    Ok(())
}

pub fn export_agent_statistics(agents: &Vec<Agent>) -> io::Result<()> {
    let mut stats = AgentStatistics {
        total_agents: agents.len(),
        agent_types: HashMap::new(),
    };

    for agent in agents {
        let agent_type = match agent.agent_type {
            AgentType::Child => "Child",
            AgentType::Teen => "Teen",
            AgentType::Adult => "Adult",
            AgentType::Elder => "Elder",
        };
        *stats.agent_types.entry(agent_type.to_string()).or_insert(0) += 1;
    }

    let json = serde_json::to_string_pretty(&stats)?;
    std::fs::write("simulation_data.json", json)?;

    Ok(())
}

/// Main Simulation struct that handles the simulation state
pub struct Simulation {
    /// The underlying simulation model
    pub model: Model,
    /// Collects agent data for analysis and visualization
    pub agent_data_collector: AgentDataCollector,
    /// Current simulation step
    pub current_step: u32,
    /// Whether tsunami has started
    pub is_tsunami: bool,
    /// Current tsunami propagation index
    pub tsunami_index: usize,
    /// Configuration for the simulation
    pub config: SimulationConfig,
}

impl Simulation {
    /// Create a new simulation with default configuration
    pub fn new(grid_path: &str, population_path: &str) -> io::Result<Self> {
        Self::with_config(grid_path, population_path, SimulationConfig::default())
    }

    /// Create a new simulation with custom configuration
    pub fn with_config(grid_path: &str, population_path: &str, config: SimulationConfig) -> io::Result<Self> {
        let (mut grid, mut agents) = load_grid_from_ascii(grid_path)?;
        let mut next_agent_id = agents.len();

        load_population_and_create_agents(
            population_path,
            grid.width,
            grid.height,
            &mut grid,
            &mut agents,
            &mut next_agent_id,
        )?;

        let model = Model {
            grid,
            agents,
            dead_agents: 0,
            dead_agent_types: Vec::new(),
        };

        Ok(Self {
            agent_data_collector: AgentDataCollector::new(model.grid.clone()),
            model,
            current_step: 0,
            is_tsunami: false,
            tsunami_index: 0,
            config,
        })
    }

    /// Builder-style method to set configuration
    pub fn with_tsunami_delay(mut self, delay: u32) -> Self {
        self.config.tsunami_delay = delay;
        self
    }

    /// Builder-style method to set tsunami speed time
    pub fn with_tsunami_speed_time(mut self, speed_time: u32) -> Self {
        self.config.tsunami_speed_time = speed_time;
        self
    }

    /// Builder-style method to set data collection interval
    pub fn with_data_collection_interval(mut self, interval: u32) -> Self {
        self.config.data_collection_interval = interval;
        self
    }

    /// Run a single simulation step, returns false when simulation should end
    pub fn step(&mut self) -> bool {
        // Return false when simulation should end
        if self.tsunami_index > self.model.grid.tsunami_data.len() - 1 {
            return false;
        }

        if self.current_step > self.config.tsunami_delay {
            self.is_tsunami = true;

            if self.current_step % self.config.tsunami_speed_time == 0 && 
               self.current_step != 0 && 
               self.is_tsunami {
                self.tsunami_index += 1;
            }
        }

        self.model.step(self.current_step, self.is_tsunami, self.tsunami_index);
        
        if self.current_step % self.config.data_collection_interval == 0 {
            self.agent_data_collector.collect_step(&self.model, self.current_step);
        }

        self.current_step += 1;
        true
    }
    
    /// Run the simulation for a specified number of steps or until completion
    pub fn run(&mut self, max_steps: Option<u32>) -> io::Result<()> {
        let mut step_count = 0;
        
        while self.step() {
            if let Some(max) = max_steps {
                step_count += 1;
                if step_count >= max {
                    break;
                }
            }
        }
        
        Ok(())
    }
} 