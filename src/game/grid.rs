use crate::game::agent::{Agent, AgentType};
use crate::game::State;
use std::collections::BinaryHeap;
use std::collections::{HashMap, VecDeque};

use serde::{Serialize, Deserialize};

/// Different types of terrain in the simulation grid
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Terrain {
    /// Blocked terrain that agents cannot traverse
    Blocked,
    /// Road terrain that agents can move on efficiently
    Road,
    /// Shelter terrain where agents can evacuate to, with shelter ID
    Shelter(u32),
    /// Custom terrain type with a traversability cost (1.0 = normal road, higher = harder to traverse)
    Custom(f64),
}

/// Configuration for the simulation grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Penalty for moving through blocked terrain
    pub blocked_penalty: u32,
    /// Whether to allow diagonal movement
    pub allow_diagonal: bool,
    /// Maximum shelter capacity (-1 for unlimited)
    pub shelter_capacity: i32,
    /// Path planning algorithm to use ("dijkstra", "a_star", "bfs")
    pub path_algorithm: String,
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig {
            blocked_penalty: 2,
            allow_diagonal: false,
            shelter_capacity: -1,
            path_algorithm: "dijkstra".to_string(),
        }
    }
}

/// Represents the simulation grid
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    /// Width of the grid in cells
    pub width: u32,
    /// Height of the grid in cells
    pub height: u32,
    /// X-coordinate of the lower-left corner in real-world coordinates
    pub xllcorner: f64,
    /// Y-coordinate of the lower-left corner in real-world coordinates
    pub yllcorner: f64,
    /// Size of each cell in real-world units
    pub cellsize: f64,
    /// 2D array of terrain types
    pub terrain: Vec<Vec<Terrain>>,
    /// List of shelter locations (x, y, shelter_id)
    pub shelters: Vec<(u32, u32, u32)>,
    /// List of agents in each cell
    pub agents_in_cell: Vec<Vec<Vec<usize>>>,
    /// Precomputed distances to nearest road
    pub distance_to_road: Vec<Vec<Option<u32>>>,
    /// Precomputed distances to nearest shelter
    pub distance_to_shelter: Vec<Vec<Option<u32>>>,
    /// Agents in each shelter
    pub shelter_agents: HashMap<u32, Vec<(usize, AgentType)>>,
    /// Population data for each cell
    pub population: Vec<Vec<u32>>,
    /// Tsunami propagation data
    pub tsunami_data: Vec<Vec<Vec<u32>>>,
    /// Number of rows (same as height)
    pub nrow: u32,
    /// Number of columns (same as width)
    pub ncol: u32,
    /// Grid configuration
    pub config: GridConfig,
}

impl Grid {
    pub fn remove_agent(&mut self, x: u32, y: u32, agent_id: usize) {
        // let cell = &mut self.agents_in_cell[y as usize][x as usize];
        // if let Some(pos) = cell.iter().position(|&id| id == agent_id) {
        //     cell.remove(pos);
        // }
        let y_usize = y as usize;
        let x_usize = x as usize;

        if y_usize < self.agents_in_cell.len() && x_usize < self.agents_in_cell[y_usize].len() {
            // Cari posisi ID agen di sel dan hapus jika ditemukan
            if let Some(index) = self.agents_in_cell[y_usize][x_usize]
                .iter()
                .position(|&id| id == agent_id)
            {
                self.agents_in_cell[y_usize][x_usize].remove(index);
            }
        }
    }

    pub fn add_to_shelter(&mut self, shelter_id: u32, agent_id: usize, agent_type: AgentType) {
        self.shelter_agents
            .entry(shelter_id)
            .or_insert(Vec::new())
            .push((agent_id, agent_type));

        // remove agent from grid
    }

    pub fn add_agent(&mut self, x: u32, y: u32, agent_id: usize) {
        self.agents_in_cell[y as usize][x as usize].push(agent_id);
    }

    /// Compute the distance from each cell to the nearest shelter
pub fn compute_distance_to_shelters(&mut self) {
        // Choose the appropriate algorithm based on configuration
        match self.config.path_algorithm.as_str() {
            "bfs" => self.compute_distance_to_shelters_bfs(),
            "a_star" => self.compute_distance_to_shelters_astar(),
            _ => self.compute_distance_to_shelters_dijkstra(), // Default to Dijkstra
        }
    }

    /// Compute shelter distances using BFS algorithm (simple but no terrain costs)
    fn compute_distance_to_shelters_bfs(&mut self) {
        let mut queue = VecDeque::new();
        let mut visited = vec![vec![false; self.width as usize]; self.height as usize];

        // Initialize all shelters with distance 0
        for &(x, y, _) in &self.shelters {
            let x = x as usize;
            let y = y as usize;
            self.distance_to_shelter[y][x] = Some(0);
            queue.push_back((x, y));
            visited[y][x] = true;
        }

        // Define movement directions (4-connected or 8-connected grid)
        let dirs = if self.config.allow_diagonal {
            // 8-connected grid (includes diagonals)
            vec![
                (0, 1), (0, -1), (1, 0), (-1, 0),  // Cardinal directions
                (1, 1), (1, -1), (-1, 1), (-1, -1)  // Diagonals
            ]
        } else {
            // 4-connected grid (cardinal directions only)
            vec![(0, 1), (0, -1), (1, 0), (-1, 0)]
        };

        while let Some((x, y)) = queue.pop_front() {
            let current_dist = self.distance_to_shelter[y][x].unwrap();

            for &(dx, dy) in &dirs {
                let nx = (x as i32) + dx;
                let ny = (y as i32) + dy;

                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    // Don't visit blocked cells or already visited cells
                    if !visited[ny][nx] && self.terrain[ny][nx] != Terrain::Blocked {
                        visited[ny][nx] = true;
                        
                        // Calculate step cost (1 for cardinal, √2 for diagonal)
                        let step_cost = if dx != 0 && dy != 0 { 2 } else { 1 };
                        self.distance_to_shelter[ny][nx] = Some(current_dist + step_cost);
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }
    
    /// Compute shelter distances using Dijkstra's algorithm (accounts for terrain costs)
    fn compute_distance_to_shelters_dijkstra(&mut self) {
        let mut dist = vec![vec![None; self.width as usize]; self.height as usize];
        let mut heap = BinaryHeap::new();

        // Initialize all shelters with distance 0
        for &(x, y, _) in &self.shelters {
            let x = x as usize;
            let y = y as usize;
            dist[y][x] = Some(0);
            heap.push(State {
                cost: 0,
                x: x as u32,
                y: y as u32,
            });
        }

        // Define movement directions (4-connected or 8-connected grid)
        let dirs = if self.config.allow_diagonal {
            // 8-connected grid (includes diagonals)
            vec![
                (0, 1), (0, -1), (1, 0), (-1, 0),  // Cardinal directions
                (1, 1), (1, -1), (-1, 1), (-1, -1)  // Diagonals
            ]
        } else {
            // 4-connected grid (cardinal directions only)
            vec![(0, 1), (0, -1), (1, 0), (-1, 0)]
        };

        // Use Dijkstra's algorithm to compute shortest paths
        while let Some(State { cost, x, y }) = heap.pop() {
            // Skip if we've found a better path already
            if let Some(current) = dist[y as usize][x as usize] {
                if cost > current {
                    continue;
                }
            }
            
            for &(dx, dy) in &dirs {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as u32;
                    let ny = ny as u32;
                    
                    // Determine cost based on terrain type
                    let extra_cost = match self.terrain[ny as usize][nx as usize] {
                        Terrain::Blocked => self.config.blocked_penalty,
                        Terrain::Road => 1,
                        Terrain::Shelter(_) => 0, // No cost for shelters
                        Terrain::Custom(cost) => cost.ceil() as u32,
                    };
                    
                    // Apply diagonal penalty if movement is diagonal
                    let diagonal_cost = if dx != 0 && dy != 0 { 
                        (1.414 * extra_cost as f64).ceil() as u32 // √2 for diagonal movement
                    } else {
                        extra_cost
                    };
                    
                    let next_cost = cost + diagonal_cost;
                    
                    // Update distance if cell not visited or shorter path found
                    if dist[ny as usize][nx as usize].is_none()
                        || next_cost < dist[ny as usize][nx as usize].unwrap()
                    {
                        dist[ny as usize][nx as usize] = Some(next_cost);
                        heap.push(State {
                            cost: next_cost,
                            x: nx,
                            y: ny,
                        });
                    }
                }
            }
        }
        
        // Store computed distances
        self.distance_to_shelter = dist;
    }
    
    /// Compute shelter distances using A* algorithm 
    /// (placeholder - would require a heuristic function)
    fn compute_distance_to_shelters_astar(&mut self) {
        // For now, just use Dijkstra - A* implementation would be similar
        // but with an added heuristic function to improve performance
        self.compute_distance_to_shelters_dijkstra();
    }

    /// Compute the distance from each cell to the nearest road
pub fn compute_road_distances_from_agents(&mut self) {
        // Initialize distance matrix with None
        let mut dist = vec![vec![None; self.width as usize]; self.height as usize];
        let mut heap = BinaryHeap::new();

        // Initialize all Road cells with distance 0 and add to heap
        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                if self.terrain[y][x] == Terrain::Road {
                    dist[y][x] = Some(0);
                    heap.push(State {
                        cost: 0,
                        x: x as u32,
                        y: y as u32,
                    });
                }
            }
        }

        // Define movement directions (4-connected or 8-connected grid)
        let dirs = if self.config.allow_diagonal {
            // 8-connected grid (includes diagonals)
            vec![
                (0, 1), (0, -1), (1, 0), (-1, 0),  // Cardinal directions
                (1, 1), (1, -1), (-1, 1), (-1, -1)  // Diagonals
            ]
        } else {
            // 4-connected grid (cardinal directions only)
            vec![(0, 1), (0, -1), (1, 0), (-1, 0)]
        };

        // Use Dijkstra's algorithm to spread distance values through the grid
        while let Some(State { cost, x, y }) = heap.pop() {
            // If current state cost is higher than stored cost, skip
            if let Some(current) = dist[y as usize][x as usize] {
                if cost > current {
                    continue;
                }
            }
            
            for &(dx, dy) in &dirs {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as u32;
                    let ny = ny as u32;
                    
                    // Determine movement cost based on terrain type
                    let extra_cost = match self.terrain[ny as usize][nx as usize] {
                        Terrain::Blocked => self.config.blocked_penalty, // Configurable penalty for blocked cells
                        Terrain::Road => 1, // Normal cost for roads
                        Terrain::Shelter(_) => 1, // Normal cost for shelters
                        Terrain::Custom(cost) => cost.ceil() as u32, // Use custom terrain cost
                    };
                    
                    // Apply diagonal penalty if movement is diagonal
                    let diagonal_cost = if dx != 0 && dy != 0 { 
                        (1.414 * extra_cost as f64).ceil() as u32 // √2 for diagonal movement
                    } else {
                        extra_cost
                    };
                    
                    let next_cost = cost + diagonal_cost;
                    
                    // Update distance if cell not visited or shorter path found
                    if dist[ny as usize][nx as usize].is_none()
                        || next_cost < dist[ny as usize][nx as usize].unwrap()
                    {
                        dist[ny as usize][nx as usize] = Some(next_cost);
                        heap.push(State {
                            cost: next_cost,
                            x: nx,
                            y: ny,
                        });
                    }
                }
            }
        }
        
        // Store computed distances
        self.distance_to_road = dist;
    }
}

/// Load a grid from an ASCII grid file with default configuration
pub fn load_grid_from_ascii(
    path: &str,
) -> Result<(Grid, Vec<crate::game::agent::Agent>), std::io::Error> {
    load_grid_from_ascii_with_config(path, GridConfig::default())
}

/// Load a grid from an ASCII grid file with custom configuration
pub fn load_grid_from_ascii_with_config(
    path: &str,
    config: GridConfig,
) -> Result<(Grid, Vec<crate::game::agent::Agent>), std::io::Error> {
    println!("Opening file {}", path);
    let content = std::fs::read_to_string(path)?;
    let mut lines = content.lines();

    // Parse header (6 lines)
    let ncols_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing ncols line"))?;
    let nrows_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing nrows line"))?;
    let xll_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing xllcorner line"))?;
    let yll_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing yllcorner line"))?;
    let cellsize_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing cellsize line"))?;
    let _nodata_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "Missing NODATA_value line")
    })?;

    // Parse header values
    let ncols: u32 = ncols_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Cannot parse ncols"))?
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid ncols value"))?;
    let nrows: u32 = nrows_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Cannot parse nrows"))?
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid nrows value"))?;
    let xllcorner: f64 = xll_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Cannot parse xllcorner"))?
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid xllcorner value"))?;
    let yllcorner: f64 = yll_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Cannot parse yllcorner"))?
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid yllcorner value"))?;
    let cellsize: f64 = cellsize_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Cannot parse cellsize"))?
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid cellsize value"))?;

    println!("Cellsize: {}", cellsize);
    println!("Nrows: {}", nrows);
    println!("Ncols: {}", ncols);
    println!("xll_line: {}", xll_line);
    println!("yll_line: {}", yll_line);
    println!("Xllcorner: {:.10}", xllcorner);
    println!("Yllcorner: {:.10}", yllcorner);

    // Initialize grid data structures
    let mut terrain = vec![vec![Terrain::Blocked; ncols as usize]; nrows as usize];
    let mut shelters = Vec::new();
    let mut agent_positions = Vec::new();
    let mut road_cells = Vec::new();

    // Parse grid data
    for (y, line) in lines.enumerate() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < ncols as usize {
            continue; // Skip incomplete lines
        }
        for (x, token) in tokens.iter().enumerate().take(ncols as usize) {
            terrain[y][x] = match *token {
                "0" | "0.0" => Terrain::Blocked,
                "1" => {
                    road_cells.push((x as u32, y as u32));
                    Terrain::Road
                }
                token if token.starts_with("20") => {
                    if let Ok(shelter_id) = token[2..].parse::<u32>() {
                        shelters.push((x as u32, y as u32, shelter_id));
                        Terrain::Shelter(shelter_id)
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Invalid shelter ID format: {}", token),
                        ));
                    }
                }
                "3" => {
                    agent_positions.push((x as u32, y as u32, AgentType::Adult));
                    Terrain::Road
                }
                "4" => {
                    // Child agent
                    agent_positions.push((x as u32, y as u32, AgentType::Child));
                    Terrain::Road
                }
                "5" => {
                    // Teen agent
                    agent_positions.push((x as u32, y as u32, AgentType::Teen));
                    Terrain::Road
                }
                "6" => {
                    // Elder agent
                    agent_positions.push((x as u32, y as u32, AgentType::Elder));
                    Terrain::Road
                }
                // Support for custom terrain with costs
                token if token.starts_with("c") => {
                    if let Ok(cost) = token[1..].parse::<f64>() {
                        Terrain::Custom(cost)
                    } else {
                        Terrain::Blocked
                    }
                }
                _ => Terrain::Blocked,
            };
        }
    }

    // Create agents from positions
    let agent_config = crate::game::agent::AgentConfig::default();
    let agents: Vec<crate::game::agent::Agent> = agent_positions
        .into_iter()
        .enumerate()
        .map(|(index, (x, y, agent_type))| {
            let is_on_road = terrain[y as usize][x as usize] == Terrain::Road;
            crate::game::agent::Agent::with_config(
                index, x, y, agent_type, is_on_road, &agent_config
            )
        })
        .collect();

    // Create and initialize the grid
    let mut grid = Grid {
        width: ncols,
        height: nrows,
        xllcorner,
        yllcorner,
        cellsize,
        terrain,
        shelters,
        agents_in_cell: vec![vec![Vec::new(); ncols as usize]; nrows as usize],
        distance_to_shelter: vec![vec![None; ncols as usize]; nrows as usize],
        shelter_agents: std::collections::HashMap::new(),
        distance_to_road: vec![vec![None; ncols as usize]; nrows as usize],
        population: vec![vec![0; ncols as usize]; nrows as usize],
        tsunami_data: Vec::new(),
        nrow: nrows,
        ncol: ncols,
        config,
    };

    // Precompute distance fields
    grid.compute_distance_to_shelters();
    grid.compute_road_distances_from_agents();

    Ok((grid, agents))
}

        