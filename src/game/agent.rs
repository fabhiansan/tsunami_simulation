#[derive(Debug, Clone, Copy)]
pub struct Agent {
    pub x: u32,
    pub y: u32,
    pub speed: u32,         // Movement probability per step (0.0-1.0)
    pub remaining_steps: u32, // Reset tiap update
    pub is_on_road: bool,
    pub search_counter: usize,
}