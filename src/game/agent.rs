#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AgentType {
    Child,
    Teen,
    Adult,
    Elder,
    Car,
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
    pub fn new(id: usize, x: u32, y: u32, agent_type: AgentType, is_on_road: bool, ) -> Self {
        let speed = match agent_type {
            AgentType::Child => 0.8 * BASE_SPEED,      // Kecepatan rendah
            AgentType::Teen => 1.0 * BASE_SPEED,      // Kecepatan lebih tinggi
            AgentType::Adult => 1.0 * BASE_SPEED,     // Kecepatan sedang 0.75 -> 1.16 m/s 
            AgentType::Elder => 0.7 * BASE_SPEED,     // Kecepatan rendah 0.4 -> 2.5 m/s == 6.25
            AgentType::Car => 1.0 * 1.68 * 5.0,
        } as u32;

        Agent {
            id, // ID akan diatur nanti
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


use rand::prelude::IndexedRandom;
use rand::{rng, thread_rng};

impl AgentType {
    pub fn random() -> Self {
        let variants = [
            AgentType::Child,
            AgentType::Teen,
            AgentType::Adult,
            AgentType::Elder,
            AgentType::Car,
        ];
        let mut rng = rng();

        *variants.choose(&mut rng).unwrap()
    }
}


use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]

pub struct DeadAgentsData {
    pub step: u32,
    pub dead_agents: usize,
}