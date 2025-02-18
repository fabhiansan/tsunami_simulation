use rand::seq::SliceRandom; // Impor trait SliceRandom

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
pub struct State {
    cost: u32,
    x: u32,
    y: u32,
}

// Implementasi Ord dan PartialOrd untuk membuat BinaryHeap berperilaku sebagai min-heap.
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Kita membalik perbandingan sehingga elemen dengan cost rendah memiliki prioritas tinggi.
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


pub mod grid;
pub mod agent;
pub mod game;