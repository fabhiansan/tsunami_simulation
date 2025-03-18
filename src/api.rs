use crate::game::agent::AgentType;
use crate::game::game::Model;
use crate::game::grid::load_grid_from_ascii;

use actix_cors::Cors;
use actix_web::{
    get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Configuration for simulation
#[derive(Serialize, Deserialize, Clone)]
pub struct SimulationConfig {
    pub location: String,
    pub grid_path: String,
    pub population_path: String,
    pub tsunami_data_path: String,
    pub output_path: String,
    pub max_steps: Option<u32>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            location: "sample".to_string(),
            grid_path: "./data_sample/sample_grid.asc".to_string(),
            population_path: "./data_sample/sample_agents.asc".to_string(),
            tsunami_data_path: "./data_sample/tsunami_ascii_sample".to_string(),
            output_path: "./output".to_string(),
            max_steps: None,
        }
    }
}

impl SimulationConfig {
    // Get location-specific paths
    pub fn get_location_paths(&self) -> (String, String, String) {
        // Use absolute project path instead of relying on current_dir
        let project_dir = "/Users/fabhiantom/San/rust/tsunami_simulation".to_string();
        println!("Using project directory: {}", project_dir);
        
        match self.location.as_str() {
            "pacitan" => (
                format!("{}/data_pacitan/jalandantes2m.asc", project_dir),
                format!("{}/data_pacitan/agent2mboundariesjalan_reprojected.asc", project_dir),
                format!("{}/data_pacitan/tsunami_ascii_pacitan", project_dir)
            ),
            "sample" => (
                format!("{}/data_sample/sample_grid.asc", project_dir),
                format!("{}/data_sample/sample_agents.asc", project_dir),
                format!("{}/data_sample/tsunami_ascii_sample", project_dir)
            ),
            _ => (
                format!("{}/data_jembrana/jalantes_jembrana.asc", project_dir),
                format!("{}/data_jembrana/agen_jembrana.asc", project_dir),
                format!("{}/data_jembrana/tsunami_ascii_jembrana", project_dir)
            ),
        }
    }
}

// Define some constants used in the simulation
const TSUNAMI_DELAY: u32 = 50; // Start tsunami earlier for testing (was 30 * 60)
const TSUNAMI_SPEED_TIME: u32 = 28;

// Current state of simulation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimulationState {
    pub current_step: u32,
    pub is_tsunami: bool,
    pub tsunami_index: usize,
    pub dead_agents: usize,
    pub is_running: bool,
    pub is_completed: bool,
}

// Agent type data for shelters
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ShelterAgentTypeData {
    pub child: u32,
    pub teen: u32,
    pub adult: u32,
    pub elder: u32,
}

// Result of simulation step
#[derive(Serialize, Deserialize)]
pub struct StepResult {
    pub step: u32,
    pub dead_agents: usize,
    pub dead_agent_types: HashMap<String, u32>,
    pub shelter_data: HashMap<String, ShelterAgentTypeData>,
}

// Application state
pub struct AppState {
    pub config: SimulationConfig,
    pub state: SimulationState,
    pub model: Option<Model>,
    pub death_json_counter: Vec<serde_json::Value>,
    pub shelter_json_counter: Vec<serde_json::Value>,
}

// Health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Tsunami Simulation API is running"
    }))
}

// Get current configuration
#[get("/config")]
async fn get_config(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let app_state = data.lock().unwrap();
    HttpResponse::Ok().json(&app_state.config)
}

// Update configuration
#[post("/config")]
async fn update_config(
    data: web::Data<Arc<Mutex<AppState>>>,
    config: web::Json<SimulationConfig>,
) -> impl Responder {
    let mut app_state = data.lock().unwrap();
    app_state.config = config.into_inner();
    
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Configuration updated"
    }))
}

// Helper function to read tsunami data from a directory
fn read_tsunami_data(_dir_path: &str, ncols: u32, nrows: u32) -> std::io::Result<Vec<Vec<Vec<u32>>>> {
    // In a real implementation, this would read tsunami data from files
    // For now, we'll create a simple mock implementation
    let mut tsunami_data = Vec::new();
    
    // Create a simple tsunami wave that progresses across the grid
    for i in 0..10 {
        let mut grid = vec![vec![0; ncols as usize]; nrows as usize];
        
        // Set tsunami height in an area that moves across the grid
        for y in 0..nrows as usize {
            for x in 0..ncols as usize {
                if x > i * (ncols as usize / 10) && x < (i + 2) * (ncols as usize / 10) {
                    grid[y][x] = 10; // Tsunami height
                }
            }
        }
        
        tsunami_data.push(grid);
    }
    
    Ok(tsunami_data)
}

// Initialize simulation
#[post("/init")]
async fn init_simulation(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let mut app_state = data.lock().unwrap();
    
    // If simulation is already running, return error
    if app_state.state.is_running {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Simulation is already running"
        }));
    }
    
    // Reset simulation state
    app_state.state = SimulationState::default();
    app_state.death_json_counter = Vec::new();
    app_state.shelter_json_counter = Vec::new();
    
    // Get location-specific paths
    let (grid_path, population_path, tsunami_data_path) = app_state.config.get_location_paths();
    
    println!("Initializing simulation for location: {}", app_state.config.location);
    println!("Using grid path: {}", grid_path);
    println!("Using population path: {}", population_path);
    println!("Using tsunami data path: {}", tsunami_data_path);
    
    // Load grid and agents from file ASC
    let grid_result = load_grid_from_ascii(&grid_path);
    if grid_result.is_err() {
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to load grid: {}", grid_result.err().unwrap())
        }));
    }
    
    // Unpack the result - load_grid_from_ascii returns (Grid, Vec<Agent>)
    let (mut grid, agents) = grid_result.unwrap();
    
    // Load tsunami data
    let tsunami_data_result = read_tsunami_data(&tsunami_data_path, grid.ncol, grid.nrow);
    if tsunami_data_result.is_err() {
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to read tsunami data: {}", tsunami_data_result.err().unwrap())
        }));
    }
    
    // Set tsunami data in grid
    grid.tsunami_data = tsunami_data_result.unwrap();
    
    // Create model
    let model = Model {
        grid,
        agents,
        dead_agents: 0,
        dead_agent_types: Vec::new(),
    };
    
    app_state.model = Some(model);
    app_state.state.is_running = true;
    app_state.state.current_step = 0;
    app_state.state.is_tsunami = false;
    app_state.state.tsunami_index = 0;
    
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Simulation initialized",
        "total_agents": app_state.model.as_ref().unwrap().agents.len()
    }))
}

// Run simulation step
#[post("/step")]
async fn run_step(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    // First, check if simulation is initialized or completed
    let initialization_check = {
        let app_state = data.lock().unwrap();
        if app_state.model.is_none() {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Simulation not initialized"
            }));
        }
        
        if app_state.state.is_completed {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Simulation already completed"
            }));
        }
        
        // Extract needed values before releasing the lock
        (
            app_state.state.current_step,
            app_state.config.max_steps
        )
    };
    
    let (current_step, max_steps) = initialization_check;
    
    // Calculate tsunami state without holding the lock
    let is_tsunami = current_step >= TSUNAMI_DELAY;
    let tsunami_index = if is_tsunami {
        ((current_step - TSUNAMI_DELAY) / TSUNAMI_SPEED_TIME) as usize
    } else {
        0
    };
    
    // Create result struct to store intermediate results
    let mut step_result = StepResult {
        step: current_step,
        dead_agents: 0,
        dead_agent_types: HashMap::new(),
        shelter_data: HashMap::new(),
    };
    
    // STEP 1: Get a clone of the model for processing
    {
        // Lock once to get the model 
        let mut app_state = data.lock().unwrap();
        
        // Update state values (safe here since we're not using model yet)
        app_state.state.is_tsunami = is_tsunami;
        app_state.state.tsunami_index = tsunami_index;
        
        // If we have a model, clone it for our use
        if let Some(model) = &mut app_state.model {
            // Run the simulation step while we have the lock
            model.step(current_step, is_tsunami, tsunami_index);
            
            // Store data for later processing
            step_result.dead_agents = model.dead_agents;
            
            // Collect dead agent types
            for agent_type in &model.dead_agent_types {
                let key = format!("{:?}", agent_type);
                *step_result.dead_agent_types.entry(key).or_insert(0) += 1;
            }
            
            // Collect shelter data
            for (&shelter_id, agents) in &model.grid.shelter_agents {
                let mut shelter_counts = ShelterAgentTypeData::default();
                for &(_, agent_type) in agents {
                    match agent_type {
                        AgentType::Child => shelter_counts.child += 1,
                        AgentType::Teen => shelter_counts.teen += 1,
                        AgentType::Adult => shelter_counts.adult += 1,
                        AgentType::Elder => shelter_counts.elder += 1,
                        _ => {} // Handle any other agent types
                    }
                }
                step_result.shelter_data.insert(format!("Shelter {}", shelter_id), shelter_counts);
            }
        }
    }
    
    // STEP 2: Update app state with results
    {
        let mut app_state = data.lock().unwrap();
        
        // Update state
        app_state.state.current_step += 1;
        app_state.state.dead_agents = step_result.dead_agents;
        
        // Create JSON data
        let death_json = json!({
            "step": current_step,
            "dead_agents": step_result.dead_agents,
            "dead_agent_types": step_result.dead_agent_types.clone()
        });
        
        let shelter_json = json!({
            "step": current_step,
            "shelters": step_result.shelter_data.clone()
        });
        
        // Add to counters
        app_state.death_json_counter.push(death_json);
        app_state.shelter_json_counter.push(shelter_json);
        
        // Check if max steps reached
        if let Some(max_steps) = max_steps {
            if current_step >= max_steps {
                app_state.state.is_completed = true;
                
                // Create copies of the data
                let death_data = app_state.death_json_counter.clone();
                let shelter_data = app_state.shelter_json_counter.clone();
                
                // Save data
                if let Some(model) = &app_state.model {
                    let _ = model.save_shelter_data(&death_data, &shelter_data);
                }
            }
        }
    }
    
    HttpResponse::Ok().json(step_result)
}

// Run multiple simulation steps
#[post("/run/{steps}")]
async fn run_steps(
    data: web::Data<Arc<Mutex<AppState>>>,
    steps: web::Path<u32>,
) -> impl Responder {
    let steps = steps.into_inner();
    let mut results = Vec::new();
    
    for _ in 0..steps {
        // Clone the data for each iteration
        let data_clone = data.clone();
        
        // Check if simulation is completed before proceeding
        let is_completed = {
            let app_state = data_clone.lock().unwrap();
            app_state.state.is_completed
        };
        
        if is_completed {
            break;
        }
        
        // Run a single step
        let step_result = run_single_step(data_clone).await;
        
        match step_result {
            Ok(result) => results.push(result),
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Failed to run step: {}", e)
                }));
            }
        }
    }
    
    HttpResponse::Ok().json(results)
}

// Helper function to run a single step
async fn run_single_step(data: web::Data<Arc<Mutex<AppState>>>) -> Result<StepResult, String> {
    // First, check if simulation is initialized or completed
    let initialization_check = {
        let app_state = data.lock().unwrap();
        if app_state.model.is_none() {
            return Err("Simulation not initialized".to_string());
        }
        
        if app_state.state.is_completed {
            return Err("Simulation already completed".to_string());
        }
        
        // Extract needed values before releasing the lock
        (
            app_state.state.current_step,
            app_state.config.max_steps
        )
    };
    
    let (current_step, max_steps) = initialization_check;
    
    // Calculate tsunami state without holding the lock
    let is_tsunami = current_step >= TSUNAMI_DELAY;
    let tsunami_index = if is_tsunami {
        ((current_step - TSUNAMI_DELAY) / TSUNAMI_SPEED_TIME) as usize
    } else {
        0
    };
    
    // Create result struct to store intermediate results
    let mut step_result = StepResult {
        step: current_step,
        dead_agents: 0,
        dead_agent_types: HashMap::new(),
        shelter_data: HashMap::new(),
    };
    
    // STEP 1: Get a clone of the model for processing
    {
        // Lock once to get the model 
        let mut app_state = data.lock().unwrap();
        
        // Update state values (safe here since we're not using model yet)
        app_state.state.is_tsunami = is_tsunami;
        app_state.state.tsunami_index = tsunami_index;
        
        // If we have a model, clone it for our use
        if let Some(model) = &mut app_state.model {
            // Run the simulation step while we have the lock
            model.step(current_step, is_tsunami, tsunami_index);
            
            // Store data for later processing
            step_result.dead_agents = model.dead_agents;
            
            // Collect dead agent types
            for agent_type in &model.dead_agent_types {
                let key = format!("{:?}", agent_type);
                *step_result.dead_agent_types.entry(key).or_insert(0) += 1;
            }
            
            // Collect shelter data
            for (&shelter_id, agents) in &model.grid.shelter_agents {
                let mut shelter_counts = ShelterAgentTypeData::default();
                for &(_, agent_type) in agents {
                    match agent_type {
                        AgentType::Child => shelter_counts.child += 1,
                        AgentType::Teen => shelter_counts.teen += 1,
                        AgentType::Adult => shelter_counts.adult += 1,
                        AgentType::Elder => shelter_counts.elder += 1,
                        _ => {} // Handle any other agent types
                    }
                }
                step_result.shelter_data.insert(format!("Shelter {}", shelter_id), shelter_counts);
            }
        }
    }
    
    // STEP 2: Update app state with results
    {
        let mut app_state = data.lock().unwrap();
        
        // Update state
        app_state.state.current_step += 1;
        app_state.state.dead_agents = step_result.dead_agents;
        
        // Create JSON data
        let death_json = json!({
            "step": current_step,
            "dead_agents": step_result.dead_agents,
            "dead_agent_types": step_result.dead_agent_types.clone()
        });
        
        let shelter_json = json!({
            "step": current_step,
            "shelters": step_result.shelter_data.clone()
        });
        
        // Add to counters
        app_state.death_json_counter.push(death_json);
        app_state.shelter_json_counter.push(shelter_json);
        
        // Check if max steps reached
        if let Some(max_steps) = max_steps {
            if current_step >= max_steps {
                app_state.state.is_completed = true;
                
                // Create copies of the data
                let death_data = app_state.death_json_counter.clone();
                let shelter_data = app_state.shelter_json_counter.clone();
                
                // Save data
                if let Some(model) = &app_state.model {
                    let _ = model.save_shelter_data(&death_data, &shelter_data);
                }
            }
        }
    }
    
    Ok(step_result)
}

// Get simulation status
#[get("/status")]
async fn get_status(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let app_state = data.lock().unwrap();
    
    // Calculate agents in shelters correctly
    let agents_in_shelters = if let Some(model) = &app_state.model {
        model.grid.shelter_agents.values().fold(0, |acc, agents| acc + agents.len())
    } else {
        0
    };
    
    HttpResponse::Ok().json(json!({
        "state": app_state.state,
        "total_agents": app_state.model.as_ref().map_or(0, |m| m.agents.len()),
        "agents_in_shelters": agents_in_shelters,
    }))
}

// Export simulation results
#[get("/export")]
async fn export_results(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let app_state = data.lock().unwrap();
    
    if app_state.model.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Simulation not initialized"
        }));
    }
    
    let model = app_state.model.as_ref().unwrap();
    
    // Extract agent data for visualization
    let agents_data: Vec<serde_json::Value> = model.agents.iter()
        .map(|agent| {
            json!({
                "id": agent.id,
                "x": agent.x,
                "y": agent.y,
                "type": format!("{:?}", agent.agent_type),
                "is_on_road": agent.is_on_road,
                "is_alive": agent.is_alive
            })
        })
        .collect();
    
    // Include tsunami data if available
    let tsunami_data = if app_state.state.is_tsunami && 
                        !model.grid.tsunami_data.is_empty() && 
                        app_state.state.tsunami_index < model.grid.tsunami_data.len() {
        Some(&model.grid.tsunami_data[app_state.state.tsunami_index])
    } else {
        None
    };
    
    // Convert tsunami data to a simplified format if available
    let tsunami_json = match tsunami_data {
        Some(data) => {
            // Convert the 2D vector to a more compact format
            // Only include cells with tsunami height > 0
            let mut tsunami_cells = Vec::new();
            for y in 0..data.len() {
                for x in 0..data[y].len() {
                    let height = data[y][x];
                    if height > 0 {
                        tsunami_cells.push(json!({
                            "x": x,
                            "y": y,
                            "height": height
                        }));
                    }
                }
            }
            tsunami_cells
        },
        None => Vec::new()
    };
    
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "step": app_state.state.current_step,
        "is_tsunami": app_state.state.is_tsunami,
        "tsunami_index": app_state.state.tsunami_index,
        "agents": agents_data,
        "tsunami_cells": tsunami_json,
        "dead_agents": model.dead_agents,
        "total_agents": model.agents.len()
    }))
}

// Reset simulation
#[post("/reset")]
async fn reset_simulation(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let mut app_state = data.lock().unwrap();
    
    app_state.state = SimulationState::default();
    app_state.model = None;
    app_state.death_json_counter = Vec::new();
    app_state.shelter_json_counter = Vec::new();
    
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Simulation reset"
    }))
}

#[get("/grid")]
/// Get the grid data for visualization
pub async fn get_grid_data(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let state = data.lock().unwrap();
    
    if let Some(model) = &state.model {
        // Return the grid data
        let grid = model.grid.clone();
        let ncol = grid.ncol;
        let nrow = grid.nrow;
        let xllcorner = grid.xllcorner;
        let yllcorner = grid.yllcorner;
        let cellsize = grid.cellsize;
        
        // Define a NODATA value (this is typically used for cells without valid data)
        let nodata_value = -9999;
        
        // Build the grid data matrix
        let mut grid_data = Vec::new();
        for y in 0..nrow {
            let mut row = Vec::new();
            for x in 0..ncol {
                // Get the terrain value at this cell
                // We'll use terrain type values for visualization
                if (y as usize) < grid.terrain.len() && (x as usize) < grid.terrain[y as usize].len() {
                    let terrain = &grid.terrain[y as usize][x as usize];
                    // Convert terrain to a numeric value for visualization
                    let cell_value = match terrain {
                        crate::game::grid::Terrain::Blocked => 0,
                        crate::game::grid::Terrain::Road => 2,
                        crate::game::grid::Terrain::Shelter(_) => 3,
                        crate::game::grid::Terrain::Custom(_) => 1,
                    };
                    row.push(cell_value);
                } else {
                    row.push(nodata_value);
                }
            }
            grid_data.push(row);
        }
        
        return HttpResponse::Ok().json(json!({
            "header": {
                "ncols": ncol,
                "nrows": nrow,
                "xllcorner": xllcorner,
                "yllcorner": yllcorner,
                "cellsize": cellsize,
                "NODATA_value": nodata_value
            },
            "grid": grid_data
        }));
    } else {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Simulation not initialized"
        }));
    }
}

// Start API server
pub async fn start_api_server(port: u16) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    
    println!("Starting API server on port {}", port);
    
    // Create app state
    let app_state = web::Data::new(Arc::new(Mutex::new(AppState {
        config: SimulationConfig::default(),
        state: SimulationState::default(),
        model: None,
        death_json_counter: Vec::new(),
        shelter_json_counter: Vec::new(),
    })));
    
    // Create and start server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5001")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["Content-Type"])
                    .max_age(3600),
            )
            .app_data(app_state.clone())
            .service(health_check)
            .service(get_config)
            .service(update_config)
            .service(init_simulation)
            .service(run_step)
            .service(run_steps)
            .service(get_status)
            .service(export_results)
            .service(reset_simulation)
            .service(get_grid_data)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
