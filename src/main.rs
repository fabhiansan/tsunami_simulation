mod game;

use game::agent::{Agent, AgentType};
use game::game::Model;
use game::grid::{load_grid_from_ascii, Grid, Terrain};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use std::fs::File;
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize)]
pub struct ShelterData {
    pub step: u32,
    pub shelters: HashMap<String, u32>,
    pub total_dead_agents: usize,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SimulationData {
    pub records: Vec<ShelterData>,
}

#[derive(Serialize, Deserialize)]
struct AgentStatistics {
    total_agents: usize,
    agent_types: HashMap<String, usize>,
}

pub fn export_agent_statistics(agents: &Vec<crate::game::agent::Agent>) -> std::io::Result<()> {
    let mut stats = AgentStatistics {
        total_agents: agents.len(),
        agent_types: HashMap::new(),
    };

    // Count agents by type
    for agent in agents {
        let agent_type = match agent.agent_type {
            crate::game::agent::AgentType::Child => "Child",
            crate::game::agent::AgentType::Teen => "Teen",
            crate::game::agent::AgentType::Adult => "Adult",
            crate::game::agent::AgentType::Elder => "Elder",
            crate::game::agent::AgentType::Car => "Car",
        };
        *stats.agent_types.entry(agent_type.to_string()).or_insert(0) += 1;
    }

    // Write to JSON file
    let json = serde_json::to_string_pretty(&stats)?;
    std::fs::write("simulation_data.json", json)?;

    Ok(())
}
pub const DISTRIBUTION_WEIGHTS: [i32; 5] = [10, 20, 30, 15, 20];

fn write_grid_to_ascii(filename: &str, model: &Model) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(filename)?;

    // Tulis header ASC dengan nilai dari grid
    writeln!(file, "ncols        {}", model.grid.width)?;
    writeln!(file, "nrows        {}", model.grid.height)?;
    writeln!(file, "xllcorner    {}", model.grid.xllcorner)?;
    writeln!(file, "yllcorner    {}", model.grid.yllcorner)?;
    writeln!(file, "cellsize     {}", model.grid.cellsize)?;
    writeln!(file, "NODATA_value  0")?;

    // Tulis data grid: tiap baris dipisahkan spasi
    for y in 0..model.grid.height as usize {
        let mut row_tokens = Vec::with_capacity(model.grid.width as usize);
        for x in 0..model.grid.width as usize {
            let token = if !model.grid.agents_in_cell[y][x].is_empty() {
                // Get the first agent in the cell (if multiple agents exist)
                let agent_id = model.grid.agents_in_cell[y][x][0];
                // Find the agent with this ID
                if let Some(agent) = model.agents.iter().find(|a| a.id == agent_id) {
                    match agent.agent_type {
                        AgentType::Child => "3",
                        AgentType::Teen => "4",
                        AgentType::Adult => "5",
                        AgentType::Elder => "6",
                        AgentType::Car => "7",
                    }
                    .to_string()
                } else {
                    // Fallback if agent not found (shouldn't happen)
                    "0".to_string()
                }
            } else {
                match model.grid.terrain[y][x] {
                    Terrain::Blocked => "0".to_string(),
                    Terrain::Road => "1".to_string(),
                    Terrain::Shelter(id) => format!("20{:02}", id),
                }
            };
            row_tokens.push(token);
        }
        let row_line = row_tokens.join(" ");
        writeln!(file, "{}", row_line)?;
    }
    Ok(())
}

use std::{fs, path};
fn main() {
    // Muat grid dan agen dari file ASC
    let (mut grid, mut agents) =
        load_grid_from_ascii("./data_pacitan/jalandantes2m.asc").expect("Failed to load grid");

    println!("grid width : {}, grid height {}", grid.width, grid.height);

    let mut next_agent_id = agents.len();

    let _ = load_population_and_create_agents(
        "./data_pacitan/agent2mboundariesjalan_reprojected.asc",
        grid.width,
        grid.height,
        &mut grid,
        &mut agents,
        &mut next_agent_id,
    )
    .expect("Failed to populate grid");

    export_agent_statistics(&agents).expect("Failed to export agent statistics");
    // match load_population_from_ascii(
    //     "./data_pacitan/agent2mboundariesjalan_reprojected.asc",
    //     grid.width,
    //     grid.height,
    // ) {
    //     Ok(population) => {
    //         grid.population = population;
    //         println!("Data populasi berhasil dimuat ke grid.");
    //     }
    //     Err(e) => eprintln!("Gagal memuat data populasi: {}", e),
    // }

    // let mut next_agent_id = agents.len();

    // // Iterasi data populasi dan tambahkan agen untuk setiap unit populasi
    // let population_data = grid.population.clone();
    // for (y, row) in population_data.iter().enumerate() {
    //     for (x, &pop) in row.iter().enumerate() {
    //         for _ in 0..pop {
    //             let is_on_road = grid.terrain[y][x] == Terrain::Road;
    //             let agent_type = crate::game::agent::AgentType::random();

    //             let mut agent = crate::game::agent::Agent::new(
    //                 next_agent_id,
    //                 x as u32,
    //                 y as u32,
    //                 agent_type,
    //                 is_on_road
    //             );
    //             agent.id = next_agent_id;
    //             agent.remaining_steps = agent.speed;
    //             agent.is_on_road = is_on_road;

    //             grid.add_agent(x as u32, y as u32, agent.id);
    //             agents.push(agent);
    //             next_agent_id += 1;
    //         }
    //     }
    // }

    // debug_write_to_ascii(population_data);
    // todo implement all tsunami data
    let tsunami_data = read_tsunami_data(
        "./data_pacitan/tsunami_pacitan/test1.asc",
        grid.width,
        grid.height,
    )
    .unwrap();
    grid.tsunami_data.push(tsunami_data);
    let tsunami_len = grid.tsunami_data.len();
    println!("Number of tsunami data {}", tsunami_len);

    fs::create_dir_all("output").expect("Gagal membuat folder output");

    let mut model = Model {
        grid,
        agents,
        dead_agents: 0,
    };

    let mut death_json_counter = Vec::new();
    let mut shelter_json_counter = Vec::new();

    let num_steps = 100;

    for step in 0..num_steps {
        let mut is_tsunami = false;
        let mut index = -1 as isize;

        if step % 10 == 0 {
            let filename = format!("output/step_{}.asc", step);
            if let Err(e) = write_grid_to_ascii(&filename, &model) {
                eprintln!("Error writing {}: {}", filename, e);
            } else {
                println!("Saved output to {}", filename);
            }

            // Save death data to JSON
            death_json_counter.push(json!({
                "step": step,
                "dead_agents": model.dead_agents
            }));

            let shelter_info: std::collections::HashMap<String, usize> = model
                .grid
                .shelters
                .iter()
                .map(|&(_x, _y, id)| {
                    // Buat key misalnya "shelter_1", "shelter_2", dst.
                    let key = format!("shelter_{}", id);
                    // Ambil jumlah agen yang sudah masuk ke shelter dengan menggunakan shelter id sebagai kunci.
                    let count = model
                        .grid
                        .shelter_agents
                        .get(&id)
                        .map(|agents| agents.len())
                        .unwrap_or(0);
                    (key, count)
                })
                .collect();

            shelter_json_counter.push(json!({
                 "step": step,
                 "shelters": shelter_info
            }));

            println!("{:?}", model.grid.shelter_agents);
        }

        if step % 4 == 0 && step != 0 {
            is_tsunami = true;

            if index > tsunami_len as isize {
                continue;
            } else {
                index += 1;
            }
        }

        model.step(step, is_tsunami, index as usize);
        println!("Step {}", step);
    }

    // Save shelter data with current dead agents count
    if let Err(e) = model.save_shelter_data(&death_json_counter, &shelter_json_counter) {
        eprintln!("Error saving shelter data: {}", e);
    }
}

pub fn load_population_and_create_agents(
    path: &str,
    ncols: u32,
    nrows: u32,
    grid: &mut Grid,
    agents: &mut Vec<crate::game::agent::Agent>,
    next_agent_id: &mut usize,
) -> std::io::Result<()> {
    // Buka file dan baca isinya
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    // Lewati 6 baris header
    for _ in 0..6 {
        lines.next();
    }

    // Baca data populasi ke dalam vector 2D
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Dimensi data populasi tidak sesuai dengan grid.",
        ));
    }

    // Simpan data populasi ke grid (jika grid memiliki field population)
    grid.population = population.clone();

    // Iterasi data populasi dan tambahkan agen untuk setiap unit populasi
    for (y, row) in population.iter().enumerate() {
        for (x, &pop) in row.iter().enumerate() {
            for _ in 0..pop {
                let is_on_road = grid.terrain[y][x] == Terrain::Road;
                let agent_type = crate::game::agent::AgentType::random();

                let mut agent = crate::game::agent::Agent::new(
                    *next_agent_id,
                    x as u32,
                    y as u32,
                    agent_type,
                    is_on_road,
                );
                // Inisialisasi lebih lanjut untuk agen
                agent.id = *next_agent_id;
                agent.remaining_steps = agent.speed;
                agent.is_on_road = is_on_road;

                // Tambahkan agen ke grid dan vektor agen
                grid.add_agent(x as u32, y as u32, agent.id);
                agents.push(agent);
                *next_agent_id += 1;
            }
        }
    }

    Ok(())
}

pub fn load_population_from_ascii(path: &str, ncols: u32, nrows: u32) -> io::Result<Vec<Vec<u32>>> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    // Lewati 6 baris header
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
            "Dimensi data populasi tidak sesuai dengan grid.",
        ));
    }

    Ok(population)
}

fn read_tsunami_data(path: &str, ncols: u32, nrows: u32) -> io::Result<Vec<Vec<u32>>> {
    println!("Loading tsunami data from {}", path);

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    // Lewati 6 baris header
    for _ in 0..6 {
        lines.next();
    }

    let mut tsunami_data: Vec<Vec<u32>> = Vec::with_capacity(nrows as usize);

    for line in lines {
        let line = line?;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < ncols as usize {
            continue;
        }
        let row: Vec<u32> = tokens
            .iter()
            .take(ncols as usize)
            // .map(|token| token.parse::<i32>().unwrap_or(0) + 1000)
            .filter_map(|token| token.parse::<f64>().ok().map(|val| (val + 1000.0) as u32))
            .collect();
        tsunami_data.push(row);
    }

    if tsunami_data.len() != nrows as usize {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Dimensi data tsunami_data tidak sesuai dengan grid.",
        ));
    }

    Ok(tsunami_data)
}

#[test]
fn test_read_tsunami_data() {
    let data = read_tsunami_data("./data_pacitan/tsunami_pacitan/test1.asc", 4324, 4332).unwrap();
    debug_write_to_ascii(data);
}

fn debug_write_to_ascii(data: Vec<Vec<u32>>) -> io::Result<()> {
    // Pastikan data tidak kosong
    if data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Data kosong"));
    }

    let nrows = data.len();
    let ncols = data[0].len();

    // Buat file output, misalnya "debug_population.asc"
    let mut file = File::create("debug_population2.asc")?;

    // Tulis header ASC (sesuai format ESRI ASCII raster)
    writeln!(file, "ncols        {}", ncols)?;
    writeln!(file, "nrows        {}", nrows)?;
    writeln!(file, "xllcorner    0")?;
    writeln!(file, "yllcorner    0")?;
    writeln!(file, "cellsize     1")?;
    writeln!(file, "NODATA_value  0")?;

    // Tulis data grid: tiap baris dipisahkan oleh baris baru
    for row in data.iter() {
        // Ubah tiap nilai dalam baris ke string dan gabungkan dengan spasi
        let row_line = row
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        writeln!(file, "{}", row_line)?;
    }

    Ok(())
}
