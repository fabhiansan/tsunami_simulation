use std::collections::{HashMap, VecDeque};
use crate::game::agent::Agent;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Terrain {
    Blocked,
    Road,
    Shelter,
}

pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub terrain: Vec<Vec<Terrain>>,
    pub shelters: Vec<(u32, u32)>,
    pub agents_in_cell: Vec<Vec<Vec<usize>>>,
    pub distance_to_shelter: Vec<Vec<Option<u32>>>,
    pub shelter_agents: HashMap<(u32, u32), Vec<usize>>,
    pub distance_to_road: Vec<Vec<Option<u32>>>,
}

impl Grid {
    pub fn add_agent(&mut self, x: u32, y: u32, agent_id: usize) {
        self.agents_in_cell[y as usize][x as usize].push(agent_id);
    }

    pub fn remove_agent(&mut self, x: u32, y: u32, agent_id: usize) {
        let cell = &mut self.agents_in_cell[y as usize][x as usize];
        if let Some(pos) = cell.iter().position(|&id| id == agent_id) {
            cell.remove(pos);
        }
    }

    pub fn compute_distance_to_shelters(&mut self) {
        let mut queue = VecDeque::new();
        let mut visited = vec![vec![false; self.width as usize]; self.height as usize];

        // Initialize all shelters with distance 0
        for &(x, y) in &self.shelters {
            let x = x as usize;
            let y = y as usize;
            self.distance_to_shelter[y][x] = Some(0);
            queue.push_back((x, y));
            visited[y][x] = true;
        }

        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        while let Some((x, y)) = queue.pop_front() {
            let current_dist = self.distance_to_shelter[y][x].unwrap();

            for &(dx, dy) in &dirs {
                let nx = (x as i32) + dx;
                let ny = (y as i32) + dy;

                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    if !visited[ny][nx] && self.terrain[ny][nx] != Terrain::Blocked {
                        visited[ny][nx] = true;
                        self.distance_to_shelter[ny][nx] = Some(current_dist + 1);
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    pub fn add_to_shelter(&mut self, x: u32, y: u32, agent_id: usize) {
        self.shelter_agents
            .entry((x, y))
            .or_insert(Vec::new())
            .push(agent_id);
    }

    pub fn is_in_shelter(&self, x: u32, y: u32) -> bool {
        self.shelter_agents.contains_key(&(x, y))
    }

    fn compute_distance_to_roads(&mut self) {
        let mut queue = VecDeque::new();
        let mut visited = vec![vec![false; self.width as usize]; self.height as usize];
        
        // Inisialisasi dari semua jalan
        for y in 0..self.height {
            for x in 0..self.width {
                if self.terrain[y as usize][x as usize] == Terrain::Road {
                    queue.push_back((x, y));
                    self.distance_to_road[y as usize][x as usize] = Some(0);
                    visited[y as usize][x as usize] = true;
                }
            }
        }

        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        
        while let Some((x, y)) = queue.pop_front() {
            let current_dist = self.distance_to_road[y as usize][x as usize].unwrap();

            for &(dx, dy) in &dirs {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    if !visited[ny as usize][nx as usize] 
                        && self.terrain[ny as usize][nx as usize] != Terrain::Blocked 
                    {
                        visited[ny as usize][nx as usize] = true;
                        self.distance_to_road[ny as usize][nx as usize] = Some(current_dist + 1);
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    fn compute_road_distances_from_agents(&mut self, agents: &[Agent]) {
        let mut queue = VecDeque::new();
        let mut visited = vec![vec![false; self.width as usize]; self.height as usize];
        
        // Inisialisasi dari semua agent yang terjebak di 0
        for agent in agents {
            // if self.terrain[agent.y as usize][agent.x as usize] != Terrain::Road {
            //     queue.push_back((agent.x, agent.y));
            //     self.distance_to_road[agent.y as usize][agent.x as usize] = Some(0);
            //     visited[agent.y as usize][agent.x as usize] = true;
            // }
            if self.terrain[agent.y as usize][agent.x as usize] == Terrain::Blocked {
                queue.push_back((agent.x, agent.y));
                self.distance_to_road[agent.y as usize][agent.x as usize] = Some(0);
                visited[agent.y as usize][agent.x as usize] = true;
            }
        }

        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        
        while let Some((x, y)) = queue.pop_front() {
            let current_dist = self.distance_to_road[y as usize][x as usize].unwrap();

            for &(dx, dy) in &dirs {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let nx = nx as u32;
                    let ny = ny as u32;
                    
                    // Prioritaskan cell dengan nilai 1 (jalan) atau 0 (bisa dilewati)
                    if !visited[ny as usize][nx as usize] 
                        && (self.terrain[ny as usize][nx as usize] == Terrain::Road 
                            || self.terrain[ny as usize][nx as usize] == Terrain::Blocked)
                    {
                        visited[ny as usize][nx as usize] = true;
                        self.distance_to_road[ny as usize][nx as usize] = Some(current_dist + 1);
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }
}

pub fn load_grid_from_ascii(path: &str) -> Result<(Grid, Vec<Agent>), std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let height = lines.len() as u32;
    let width = lines[0].len() as u32;

    let mut terrain = vec![vec![Terrain::Blocked; width as usize]; height as usize];
    let mut shelters = Vec::new();
    let mut agent_positions = Vec::new();
    let mut road_cells = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            terrain[y][x] = match c {
                '0' => Terrain::Blocked,
                '1' => {
                    road_cells.push((x as u32, y as u32));
                    Terrain::Road
                },
                '2' => {
                    agent_positions.push((x as u32, y as u32));
                    Terrain::Road
                }
                '3' => {
                    shelters.push((x as u32, y as u32));
                    Terrain::Shelter
                }
                _ => Terrain::Blocked,
            };
        }
    }

    let mut grid = Grid {
        width,
        height,
        terrain,
        shelters,
        agents_in_cell: vec![vec![Vec::new(); width as usize]; height as usize],
        distance_to_shelter: vec![vec![None; width as usize]; height as usize],
        shelter_agents: HashMap::new(),
        distance_to_road: vec![vec![None; width as usize]; height as usize], // Inisialisasi di sini
    };
    
    // Setelah membuat grid
    grid.compute_distance_to_shelters();

    let agents: Vec<Agent> = agent_positions
        .into_iter()
        .map(|(x, y)| {
            let is_on_road = grid.terrain[y as usize][x as usize] == Terrain::Road;
            Agent {
                x,
                y,
                speed: 4,
                remaining_steps: 4,
                is_on_road,
                search_counter: 0,
            }
        })
        .collect();

    grid.compute_road_distances_from_agents(&agents);  // Gunakan fungsi baru
    
    
    Ok((grid, agents))
}
