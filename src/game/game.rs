use std::collections::VecDeque;
use std::time::Instant;

use ggez::event::{EventHandler, MouseButton};
use ggez::glam::{Affine2, Vec2};
use ggez::graphics::{DrawParam, Mesh};
use ggez::mint::{ColumnMatrix4, Vector4};
use ggez::{graphics, mint, Context, GameResult};
use rand::seq::SliceRandom;
use std::collections::HashSet;

use crate::game::{
    agent::Agent,
    camera::Camera,
    grid::{Grid, Terrain},
};

pub struct Model {
    pub grid: Grid,
    pub agents: Vec<Agent>,
    pub cell_size: f32,
    pub camera: Camera,
}

impl Model {
    fn step(&mut self) {
        let mut rng = rand::rng();
        let mut agent_order: Vec<usize> = (0..self.agents.len()).collect();

        // Reset remaining steps
        for agent in &mut self.agents {
            agent.remaining_steps = agent.speed;
        }

        for _ in 0..self.agents.iter().map(|a| a.speed).max().unwrap_or(1) {
            agent_order.shuffle(&mut rng);
            let mut reserved_cells = HashSet::new();
            let mut moves = Vec::new();

            // First pass: Kumpulkan gerakan
            for &id in &agent_order {
                let agent = &self.agents[id];
                if agent.remaining_steps == 0 || self.grid.is_in_shelter(agent.x, agent.y) {
                    continue;
                }

                if let Some((nx, ny)) = self.find_best_move(agent, &reserved_cells) {
                    reserved_cells.insert((nx, ny));
                    moves.push((id, nx, ny));
                }
            }

            // Second pass: Eksekusi gerakan
            for &(id, new_x, new_y) in &moves {
                let agent = &mut self.agents[id];
                self.grid.remove_agent(agent.x, agent.y, id);

                let was_on_road = agent.is_on_road;
                agent.is_on_road =
                    self.grid.terrain[new_y as usize][new_x as usize] == Terrain::Road;

                // Jika baru sampai di jalan
                if !was_on_road && agent.is_on_road {
                    println!("Agent {} reached road at ({}, {})", id, new_x, new_y);
                }

                agent.x = new_x;
                agent.y = new_y;
                agent.remaining_steps -= 1;

                // Jika masuk shelter
                if self.grid.terrain[new_y as usize][new_x as usize] == Terrain::Shelter {
                    self.grid.add_to_shelter(new_x, new_y, id);
                    agent.remaining_steps = 0; // Hentikan pergerakan
                } else {
                    self.grid.add_agent(new_x, new_y, id);
                }
            }
        }
    }

    fn find_best_move(&self, agent: &Agent, reserved: &HashSet<(u32, u32)>) -> Option<(u32, u32)> {
        let mut candidates = Vec::new();
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        // Jika agent di area 0
        // if self.grid.terrain[agent.y as usize][agent.x as usize] == Terrain::Blocked {
        //     // Cari gerakan yang mengurangi jarak ke jalan terdekat
        //     for &(dx, dy) in &dirs {
        //         let nx = agent.x as i32 + dx;
        //         let ny = agent.y as i32 + dy;

        //         if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
        //         {
        //             let nx = nx as u32;
        //             let ny = ny as u32;

        //             if self.grid.terrain[ny as usize][nx as usize] != Terrain::Blocked
        //                 && !reserved.contains(&(nx, ny))
        //                 && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
        //             {
        //                 if let (Some(current_dist), Some(new_dist)) = (
        //                     self.grid.distance_to_road[agent.y as usize][agent.x as usize],
        //                     self.grid.distance_to_road[ny as usize][nx as usize],
        //                 ) {
        //                     if new_dist < current_dist {
        //                         candidates.push((new_dist, nx, ny));
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     // Jika tidak ada gerakan valid, cari jalan alternatif
        //     if candidates.is_empty() {
        //         return self.find_emergency_escape(agent.x, agent.y, reserved);
        //     }
        // }

        if self.grid.terrain[agent.y as usize][agent.x as usize] == Terrain::Blocked {
            // Cari gerakan yang mengurangi jarak ke jalan terdekat
            for &(dx, dy) in &dirs {
                let nx = agent.x as i32 + dx;
                let ny = agent.y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    if self.grid.terrain[ny as usize][nx as usize] != Terrain::Blocked
                        && !reserved.contains(&(nx, ny))
                        && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                    {
                        if let (Some(current_dist), Some(new_dist)) = (
                            self.grid.distance_to_road[agent.y as usize][agent.x as usize],
                            self.grid.distance_to_road[ny as usize][nx as usize],
                        ) {
                            if new_dist < current_dist {
                                candidates.push((new_dist, nx, ny));
                            }
                        }
                    }
                }
            }

            // Jika masih tidak ada gerakan valid, gunakan strategi darurat
            if candidates.is_empty() {
                return self.find_emergency_escape(agent.x, agent.y, reserved);
            }
        }

        if agent.search_counter > 0 {
            return self.find_emergency_route(agent.x, agent.y, reserved);
        }

        let mut candidates = Vec::new();
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        // Prioritas 1: Jika belum di jalan, cari jalan terdekat
        if !agent.is_on_road {
            for &(dx, dy) in &dirs {
                let nx = agent.x as i32 + dx;
                let ny = agent.y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    if self.grid.terrain[ny as usize][nx as usize] != Terrain::Blocked
                        && !reserved.contains(&(nx, ny))
                        && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                    {
                        if let Some(current_dist) =
                            self.grid.distance_to_road[agent.y as usize][agent.x as usize]
                        {
                            if let Some(new_dist) =
                                self.grid.distance_to_road[ny as usize][nx as usize]
                            {
                                if new_dist < current_dist {
                                    candidates.push((new_dist, nx, ny));
                                }
                            }
                        }
                    }
                }
            }
        }
        // Prioritas 2: Jika sudah di jalan, cari shelter
        else {
            for &(dx, dy) in &dirs {
                let nx = agent.x as i32 + dx;
                let ny = agent.y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    let is_shelter =
                        self.grid.terrain[ny as usize][nx as usize] == Terrain::Shelter;
                    let is_blocked =
                        self.grid.terrain[ny as usize][nx as usize] == Terrain::Blocked;
                    let is_reserved = reserved.contains(&(nx, ny));
                    let has_agents = !self.grid.agents_in_cell[ny as usize][nx as usize].is_empty();

                    if is_shelter {
                        candidates.push((0, nx, ny));
                    } else if !is_blocked && !is_reserved && !has_agents {
                        if let Some(dist) = self.grid.distance_to_shelter[ny as usize][nx as usize]
                        {
                            candidates.push((dist, nx, ny));
                        }
                    }
                }
            }
        }

        candidates.sort_by_key(|&(d, _, _)| d);
        candidates.first().map(|&(_, x, y)| (x, y))
    }

    fn find_emergency_escape(
        &self,
        x: u32,
        y: u32,
        reserved: &HashSet<(u32, u32)>,
    ) -> Option<(u32, u32)> {
        let mut best_move = None;
        let mut min_distance = u32::MAX;
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dx, dy) in &dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32 {
                let nx = nx as u32;
                let ny = ny as u32;

                if self.grid.terrain[ny as usize][nx as usize] != Terrain::Blocked
                    && !reserved.contains(&(nx, ny))
                    && self.grid.agents_in_cell[ny as usize][nx as usize].is_empty()
                {
                    if let Some(dist) = self.grid.distance_to_road[ny as usize][nx as usize] {
                        if dist < min_distance {
                            min_distance = dist;
                            best_move = Some((nx, ny));
                        }
                    }
                }
            }
        }

        best_move
    }

    fn find_emergency_route(
        &self,
        x: u32,
        y: u32,
        reserved: &HashSet<(u32, u32)>,
    ) -> Option<(u32, u32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((x, y));

        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        while let Some((cx, cy)) = queue.pop_front() {
            for &(dx, dy) in &dirs {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < self.grid.width as i32 && ny < self.grid.height as i32
                {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    if self.grid.terrain[ny as usize][nx as usize] == Terrain::Road {
                        return Some((nx, ny));
                    }

                    if !visited.contains(&(nx, ny))
                        && self.grid.terrain[ny as usize][nx as usize] != Terrain::Blocked
                    {
                        visited.insert((nx, ny));
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        None
    }
}

impl EventHandler for Model {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(2) {
            self.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);

        let transform_affine = Affine2::IDENTITY
            * Affine2::from_translation(Vec2::new(
                self.camera.offset_x as f32,
                self.camera.offset_y as f32,
            ))
            * Affine2::from_scale(Vec2::new(self.camera.zoom as f32, self.camera.zoom as f32));

        let m = transform_affine.to_cols_array();

        let transform_matrix = ColumnMatrix4 {
            x: Vector4 {
                x: m[0],
                y: m[1],
                z: 0.0,
                w: 0.0,
            },
            y: Vector4 {
                x: m[2],
                y: m[3],
                z: 0.0,
                w: 0.0,
            },
            z: Vector4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 0.0,
            },
            w: Vector4 {
                x: m[4],
                y: m[5],
                z: 0.0,
                w: 1.0,
            },
        };

        let mut quad_mesh_builder = graphics::MeshBuilder::new();
        // Draw grid cells
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let color = match self.grid.terrain[y as usize][x as usize] {
                    Terrain::Blocked => graphics::Color::BLACK,
                    Terrain::Road => graphics::Color::new(0.8, 0.8, 0.8, 1.0),
                    Terrain::Shelter => graphics::Color::GREEN,
                };

                let rect = graphics::Rect::new(
                    x as f32 * self.cell_size,
                    y as f32 * self.cell_size,
                    self.cell_size,
                    self.cell_size,
                );
                let _ = quad_mesh_builder.rectangle(graphics::DrawMode::stroke(2.0), rect, color);
            }
        }

        let mesh_data = quad_mesh_builder.build();
        let mesh = Mesh::from_data(ctx, mesh_data);

        canvas.draw(&mesh, DrawParam::default().transform(transform_matrix));

        // Draw agents
        for agent in &self.agents {
            // let color = if agent.moved {
            //     graphics::Color::RED
            // } else {
            //     graphics::Color::BLUE
            // };
            let color = graphics::Color::RED;

            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2 {
                    x: agent.x as f32 * self.cell_size + self.cell_size / 2.0,
                    y: agent.y as f32 * self.cell_size + self.cell_size / 2.0,
                },
                self.cell_size / 3.0,
                0.1,
                color,
            )?;
            canvas.draw(
                &circle,
                graphics::DrawParam::default().transform(transform_matrix),
            );
        }

        canvas.finish(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        // println!("mouse pressed");
        if button == MouseButton::Left {
            // if x >= 10.0 && x <= 50.0 && y >= 10.0 && y <= 50.0 {
            //     self.is_playing = true;
            //     self.step_requested = false; // Reset step request
            //     println!("Play button clicked");
            // }

            // // Pause button
            // if x >= 60.0 && x <= 100.0 && y >= 10.0 && y <= 50.0 {
            //     self.is_playing = false;
            //     println!("Pause button clicked");
            // }

            // // Step button
            // if x >= 110.0 && x <= 150.0 && y >= 10.0 && y <= 50.0 {
            //     self.step_requested = true;
            //     println!("Step button clicked");
            // }

            let now = Instant::now();

            if let Some(last_click) = self.camera.last_click_time {
                if now.duration_since(last_click).as_millis() < 300 {
                    println!("double clicked at {} {}", x, y);
                    self.camera.zoom = 1.0;
                    self.camera.offset_x = 0.0;
                    self.camera.offset_y = 0.0;
                }
            }

            self.camera.last_click_time = Some(now);

            self.camera.dragging = true;
            self.camera.drag_start_x = x as f64;
            self.camera.drag_start_y = y as f64;
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        if button == MouseButton::Left {
            self.camera.dragging = false;
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        self.camera.mouse_x = x as f64;
        self.camera.mouse_y = y as f64;

        if self.camera.dragging {
            // println!("dragging");

            let delta_x = (x as f64 - self.camera.drag_start_x) * (self.camera.zoom + 1.0);
            let delta_y = (y as f64 - self.camera.drag_start_y) * (self.camera.zoom + 1.0);

            self.camera.offset_x += delta_x as f64 / self.camera.zoom;
            self.camera.offset_y += delta_y as f64 / self.camera.zoom;

            self.camera.drag_start_x = x as f64;
            self.camera.drag_start_y = y as f64;
        }
        Ok(())
    }

    fn mouse_wheel_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        let zoom_factor = 1.0 + (y as f64 * 0.1);
        let prev_zoom = self.camera.zoom;

        self.camera.zoom *= zoom_factor;

        // Hitung posisi dunia sebelum zoom
        let world_mouse_x = (self.camera.mouse_x - self.camera.offset_x) / prev_zoom;
        let world_mouse_y = (self.camera.mouse_y - self.camera.offset_y) / prev_zoom;

        // Hitung posisi dunia setelah zoom dan sesuaikan offset
        self.camera.offset_x = self.camera.mouse_x - world_mouse_x * self.camera.zoom;
        self.camera.offset_y = self.camera.mouse_y - world_mouse_y * self.camera.zoom;

        Ok(())
    }
}
