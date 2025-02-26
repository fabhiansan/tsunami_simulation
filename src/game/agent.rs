use rand::prelude::*;
use rand::distributions::WeightedIndex;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AgentType {
    Child,
    Teen,
    Adult,
    Elder,
}

#[derive(Debug)]
pub struct Agent {
    pub id: usize,
    pub x: u32,
    pub y: u32,
    pub speed: u32,
    pub remaining_steps: u32,
    pub is_on_road: bool,
    pub agent_type: AgentType,
    pub is_alive: bool
}

pub const BASE_SPEED: f64 = 2.66;

impl Agent {
    pub fn new(id: usize, x: u32, y: u32, agent_type: AgentType, is_on_road: bool) -> Self {
        let speed = match agent_type {
            AgentType::Child => 0.8 * BASE_SPEED,      // Kecepatan rendah
            AgentType::Teen => 1.0 * BASE_SPEED,      // Kecepatan lebih tinggi
            AgentType::Adult => 1.0 * BASE_SPEED,     // Kecepatan sedang 0.75 -> 1.16 m/s 
            AgentType::Elder => 0.7 * BASE_SPEED,     // Kecepatan rendah 0.4 -> 2.5 m/s == 6.25
        } as u32;

        Agent {
            id,
            x,
            y,
            speed,
            remaining_steps: speed,
            is_on_road,
            agent_type,
            is_alive: true
        }
    }
}

impl AgentType {
    pub fn random() -> Self {
        let weights = [6.21, 13.41, 59.10, 19.89]; // Distribusi bobot
        let variants = [
            AgentType::Child,
            AgentType::Teen,
            AgentType::Adult,
            AgentType::Elder,
        ];

        let mut rng = thread_rng();
        let dist = WeightedIndex::new(&weights).unwrap();
        variants[dist.sample(&mut rng)]
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeadAgentsData {
    pub step: u32,
    pub dead_agents: usize,
}