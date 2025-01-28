use ggez::GameResult;
use ggez::{self, event};
mod game;

use crate::game::{
    agent::Agent,
    camera::Camera,
    game::Model,
    grid::{load_grid_from_ascii, Terrain},
};

pub const UPDATE_TIME: f64 = 0.1;
pub const DELAY: u128 = 10;
pub const ARRIVAL_TIME: u128 = 30;
pub const STEP_TO_SAVE: u128 = 30;
pub const DISTRIBUTION_WEIGHTS: [i32; 6] = [10, 20, 5, 30, 15, 20];

fn main() -> GameResult {
    let (grid, agents) = load_grid_from_ascii("map.txt").expect("Failed to load grid");
    // let agents = agents
    //     .into_iter()
    //     .map(|(x, y)| {
    //         // let is_on_road = grid.terrain[y as usize][x as usize] == Terrain::Road;

    //         let mut is_near_road = false;
    //         let dirs = [
    //             (-1, -1),
    //             (0, -1),
    //             (1, -1),
    //             (-1, 0),
    //             (1, 0),
    //             (-1, 1),
    //             (0, 1),
    //             (1, 1),
    //         ];

    //         for &(dx, dy) in &dirs {
    //             let nx = x as i32 + dx;
    //             let ny = y as i32 + dy;

    //             if nx >= 0 && ny >= 0 && nx < grid.width as i32 && ny < grid.height as i32 {
    //                 if grid.terrain[ny as usize][nx as usize] == Terrain::Road {
    //                     is_near_road = true;
    //                     break;
    //                 }
    //             }
    //         }

    //         Agent {
    //             x,
    //             y,
    //             speed: 4,
    //             remaining_steps: 4,
    //             is_on_road: is_near_road, // Set true jika ada jalan di sekitar
    //             search_counter: 0,        // Tambahkan field untuk tracking pencarian
    //         }
    //     })
    //     .collect();

    println!("{:?}", agents);

    let model = Model {
        grid,
        agents,
        cell_size: 10.0,
        camera: Camera::new(),
    };

    let (ctx, event_loop) = ggez::ContextBuilder::new("tsunami_abm", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Tsunami Evacuation"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .resizable(true)
                .dimensions(
                    model.grid.width as f32 * model.cell_size,
                    model.grid.height as f32 * model.cell_size,
                ),
        )
        .build()?;

    event::run(ctx, event_loop, model)
}
