use super::agent::{Agent, DeadAgentsData};
use super::grid::{Grid, Terrain};
use crate::ShelterData;
use crate::SimulationData;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;

pub struct Model {
    pub grid: Grid,
    pub agents: Vec<Agent>,
    pub dead_agents: usize,
}

impl Model {
    pub fn step(&mut self, step: u32, is_tsunami: bool, tsunami_number: usize) {
        let mut dead_agents_this_step = 0;

        if is_tsunami {
            println!("TSUNAMI IS COMMING ");
            let tsunami_data = self.grid.tsunami_data[tsunami_number].clone();

            // TODO Implement tsunami to calculate who die
            for i in (0..self.agents.len()).rev() {
                let agent = &self.agents[i];
                // Pastikan koordinat agen tidak melebihi batas matriks
                if (agent.y as usize) < tsunami_data.len()
                    && (agent.x as usize) < tsunami_data[0].len()
                {
                    // Jika nilai pada posisi agen bukan 0, berarti agen tersebut terkena tsunami
                    let tsunami_height = tsunami_data[agent.y as usize][agent.x as usize];
                    if tsunami_height > 0 {
                        dead_agents_this_step += 1;
                        // Hapus agen dari grid (misalnya, jika grid menyimpan referensi agen per sel)
                        self.grid.remove_agent(agent.x, agent.y, i);
                        println!(
                            "Agent {} mati akibat tsunami pada koordinat ({}, {})",
                            i, agent.x, agent.y
                        );

                        // Hapus agen dari daftar agen
                        self.agents.remove(i);
                    }
                }
            }
            println!("Jumlah agen mati pada step ini: {}", dead_agents_this_step);

            for row in self.grid.tsunami_data[tsunami_number].iter_mut() {
                for cell in row.iter_mut() {
                    if *cell != 0 {
                        *cell = 10;
                    }
                }
            }
        }

        // if let Err(e) = self.save_dead_agents_data(step, dead_agents_this_step) {
        //     eprintln!("Gagal menyimpan data agen mati: {}", e);
        // }

        // Update total dead agents
        self.dead_agents += dead_agents_this_step;

        let mut rng = rand::thread_rng();
        let mut agent_order: Vec<usize> = (0..self.agents.len()).collect();

        // Reset remaining_steps untuk setiap agen
        for agent in &mut self.agents {
            agent.remaining_steps = agent.speed;
        }

        // Lakukan pergerakan sesuai jumlah langkah maksimum agen
        for _ in 0..self.agents.iter().map(|a| a.speed).max().unwrap_or(1) {
            agent_order.shuffle(&mut rng);
            let mut reserved_cells = HashSet::new();
            let mut moves = Vec::new();

            // Pass 1: Kumpulkan pergerakan
            for &id in &agent_order {
                let agent = &self.agents[id];
                if agent.remaining_steps == 0 || self.is_in_shelter(agent.x, agent.y) {
                    continue;
                }
                if let Some((nx, ny, fallback)) = self.find_best_move(agent, &reserved_cells) {
                    reserved_cells.insert((nx, ny));
                    moves.push((id, nx, ny, fallback));
                }
            }

            // Pass 2: Eksekusi pergerakan
            for &(id, new_x, new_y, fallback) in &moves {
                let (old_x, old_y) = {
                    let agent = &self.agents[id];
                    (agent.x, agent.y)
                };

                self.grid.remove_agent(old_x, old_y, id);

                {
                    let agent = &mut self.agents[id];
                    let was_on_road = agent.is_on_road;
                    agent.is_on_road =
                        self.grid.terrain[new_y as usize][new_x as usize] == Terrain::Road;

                    if !was_on_road && agent.is_on_road {
                        // println!("Agent {} reached road at ({}, {})", id, new_x, new_y);
                    }

                    agent.x = new_x;
                    agent.y = new_y;

                    // Jika gerakan fallback, kurangi remaining_steps dua kali lipat
                    if fallback {
                        if agent.remaining_steps >= 2 {
                            agent.remaining_steps -= 2;
                        } else {
                            agent.remaining_steps = 0;
                        }
                    } else {
                        agent.remaining_steps -= 1;
                    }

                    // todo change add to shelter to receive value of grid instead of their coordinates
                    // if matches!(
                    //     self.grid.terrain[new_y as usize][new_x as usize],
                    //     Terrain::Shelter(_)
                    // ) {
                    //     self.grid.add_to_shelter( , id);
                    //     agent.remaining_steps = 0;
                    // } else {
                    //     self.grid.add_agent(new_x, new_y, id);
                    // }
                    if let Terrain::Shelter(shelter_id) =
                        self.grid.terrain[new_y as usize][new_x as usize]
                    {
                        self.grid.add_to_shelter(shelter_id, id);
                        agent.remaining_steps = 0;
                    } else {
                        self.grid.add_agent(new_x, new_y, id);
                    }
                }
            }
        }
    }

    fn find_best_move(
        &self,
        agent: &Agent,
        reserved: &HashSet<(u32, u32)>,
    ) -> Option<(u32, u32, bool)> {
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut candidates = Vec::new();

        // Jika agen tidak berada di Road, cari jalan terdekat
        if self.grid.terrain[agent.y as usize][agent.x as usize] != Terrain::Road {
            if let Some(current_dist) =
                self.grid.distance_to_road[agent.y as usize][agent.x as usize]
            {
                for &(dx, dy) in &dirs {
                    let nx = agent.x as i32 + dx;
                    let ny = agent.y as i32 + dy;

                    if nx >= 0
                        && ny >= 0
                        && nx < self.grid.width as i32
                        && ny < self.grid.height as i32
                    {
                        let nx = nx as u32;
                        let ny = ny as u32;

                        if !reserved.contains(&(nx, ny))
                            && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                        {
                            // if let Some(new_dist) =
                            //     self.grid.distance_to_road[ny as usize][nx as usize]
                            // {
                            //     if new_dist < current_dist {
                            //         candidates.push((new_dist, nx, ny));
                            //     }
                            // }
                            if let Some(new_dist) =
                                self.grid.distance_to_road[ny as usize][nx as usize]
                            {
                                if new_dist <= current_dist {
                                    candidates.push((new_dist, nx, ny));
                                }
                            }
                        }
                    }
                }
            }

            if !candidates.is_empty() {
                candidates.sort_by_key(|&(d, _, _)| d);
                let (_, nx, ny) = candidates[0];
                return Some((nx, ny, false));
            }
        }
        // Jika agen berada di Road, cari shelter terdekat
        else if self.grid.terrain[agent.y as usize][agent.x as usize] == Terrain::Road {
            for &(dx, dy) in &dirs {
                let nx = agent.x as i32 + dx;
                let ny = agent.y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    // Prioritaskan jika ditemukan shelter
                    if matches!(
                        self.grid.terrain[ny as usize][nx as usize],
                        Terrain::Shelter(_)
                    ) && !reserved.contains(&(nx, ny))
                    {
                        return Some((nx, ny, false));
                    }

                    // Jika tidak, cari jalan menuju shelter melalui Road
                    if (self.grid.terrain[ny as usize][nx as usize] == Terrain::Road
                        || matches!(
                            self.grid.terrain[ny as usize][nx as usize],
                            Terrain::Shelter(_)
                        ))
                        && !reserved.contains(&(nx, ny))
                        && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                    {
                        if let Some(dist) = self.grid.distance_to_shelter[ny as usize][nx as usize]
                        {
                            candidates.push((dist, nx, ny));
                        }
                    }
                }
            }

            if !candidates.is_empty() {
                candidates.sort_by_key(|&(d, _, _)| d);
                let (_, nx, ny) = candidates[0];
                return Some((nx, ny, false));
            }
        }

        // Jika tidak ada pilihan lain, lakukan fallback move
        let fallback_moves: Vec<(u32, u32)> = dirs
            .iter()
            .filter_map(|&(dx, dy)| {
                let nx = agent.x as i32 + dx;
                let ny = agent.y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;
                    if !reserved.contains(&(nx, ny))
                        && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                    {
                        Some((nx, ny))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !fallback_moves.is_empty() {
            let mut rng = rand::thread_rng();
            let chosen = fallback_moves.choose(&mut rng).unwrap();
            Some((chosen.0, chosen.1, true))
        } else {
            None
        }
    }

    pub fn is_in_shelter(&self, x: u32, y: u32) -> bool {
        matches!(
            self.grid.terrain[y as usize][x as usize],
            Terrain::Shelter(_)
        )
    }

    pub fn save_shelter_data(
        &self,
        death_json_counter: &Vec<serde_json::Value>,
        shelter_json_counter: &Vec<serde_json::Value>,
    ) -> std::io::Result<()> {
        let filename = "output/shelter_data.json";

        // Gabungkan semua data ke dalam satu objek JSON
        let data = json!({
            "death_json_counter": death_json_counter,
            "shelter_json_counter": shelter_json_counter
        });

        // Tulis data ke file dengan format pretty JSON
        let file = File::create(filename)?;
        serde_json::to_writer_pretty(file, &data)?;
        println!("Updated simulation data in {}", filename);

        Ok(())
    }

    // pub fn save_shelter_data(&self, step: u32, total_dead_agents: usize) -> std::io::Result<()> {
    //     let filename = "output/shelter_data.json";

    //     // Read existing data or create new
    //     let mut simulation_data = if let Ok(file) = File::open(filename) {
    //         serde_json::from_reader(file).unwrap_or_else(|_| SimulationData::default())
    //     } else {
    //         SimulationData::default()
    //     };

    //     // Create new record for current step
    //     let mut shelter_data = ShelterData {
    //         step,
    //         shelters: HashMap::new(),
    //         total_dead_agents,
    //     };

    //     // Get previous shelter counts or start with 0
    //     let prev_shelters = if let Some(last_record) = simulation_data.records.last() {
    //         last_record.shelters.clone()
    //     } else {
    //         HashMap::new()
    //     };

    //     // Update shelter counts (accumulate from previous step)
    //     for &(x, y, id) in &self.grid.shelters {
    //         let shelter_key = format!("shelter_{}", id);
    //         let current_count = if let Some(agents) = self.grid.shelter_agents.get(&(x, y)) {
    //             agents.len() as u32
    //         } else {
    //             0
    //         };
    //         let prev_count = prev_shelters.get(&shelter_key).copied().unwrap_or(0);
    //         shelter_data.shelters.insert(shelter_key, prev_count + current_count);
    //     }

    //     // Add new record
    //     simulation_data.records.push(shelter_data);

    //     // Write back to file
    //     let file = File::create(filename)?;
    //     serde_json::to_writer_pretty(file, &simulation_data)?;
    //     println!("Updated simulation data in {}", filename);

    //     Ok(())
    // }

    // pub fn save_dead_agents_data(&self, step: u32, dead_agents: usize) -> std::io::Result<()> {
    //     let dead_agents_data = DeadAgentsData { step, dead_agents };
    //     let filename = format!("output/dead_agents_data_{}.json", step);
    //     let file = std::fs::File::create(&filename)?;

    //     serde_json::to_writer_pretty(file, &dead_agents_data)?;
    //     println!("Saved dead agents data to {}", filename);

    //     Ok(())
    // }
}
