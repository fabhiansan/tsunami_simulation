// use ggez::event::{EventHandler, MouseButton};
// use ggez::glam;
// use ggez::{event, graphics, mint, timer, Context, GameResult};
// use mint::Point2;
use std::time::Instant;

#[derive(Debug)]
pub struct Camera {
    pub zoom: f64,
    pub offset_x: f64, // camera control x
    pub offset_y: f64, // camera control y
    pub dragging: bool,
    pub drag_start_x: f64,
    pub drag_start_y: f64,
    pub last_click_time: Option<Instant>,
    pub mouse_x: f64,
    pub mouse_y: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            last_click_time: None,
            mouse_x: 0.0, 
            mouse_y: 0.0,
        }
    }
}

// impl Camera {
//     pub fn new() -> Self {
//         Camera {
//             offset: Point2 { x: 0.0, y: 0.0 },
//             scale: 1.0,
//             drag_start: None,
//         }
//     }

//     pub fn matrix(&self) -> graphics::Transform {
//         // Membuat matriks transformasi yang benar
//         let scale_matrix = glam::Mat4::from_scale(glam::Vec3::new(self.scale, self.scale, 1.0));
//         let translation_matrix =
//             glam::Mat4::from_translation(glam::Vec3::new(self.offset.x, self.offset.y, 0.0));

//         // Gabungkan transformasi: scale kemudian translate
//         scale_matrix * translation_matrix
//     }

//     pub fn world_to_screen(&self, point: (f32, f32)) -> (f32, f32) {
//         (
//             point.0 * self.scale + self.offset.x,
//             point.1 * self.scale + self.offset.y,
//         )
//     }

//     pub fn screen_to_world(&self, point: (f32, f32)) -> (f32, f32) {
//         (
//             (point.0 - self.offset.x) / self.scale,
//             (point.1 - self.offset.y) / self.scale,
//         )
//     }
// }
