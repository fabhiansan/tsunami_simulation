use rand::prelude::*;
use rand::distributions::WeightedIndex;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Represents different types of agents in the simulation
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Child,
    Teen,
    Adult,
    Elder,
    /// Custom agent type with specific speed multiplier
    Custom(f64),
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentType::Child => write!(f, "Child"),
            AgentType::Teen => write!(f, "Teen"),
            AgentType::Adult => write!(f, "Adult"),
            AgentType::Elder => write!(f, "Elder"),
            AgentType::Custom(multiplier) => write!(f, "Custom({})", multiplier),
        }
    }
}

/// Configuration for agent behavior and characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Base movement speed in meters per second
    pub base_speed: f64,
    /// Speed multipliers for different agent types [Child, Teen, Adult, Elder]
    pub speed_multipliers: [f64; 4],
    /// Distribution weights for agent types [Child, Teen, Adult, Elder]
    pub type_weights: [f64; 4],
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            base_speed: 2.66,
            speed_multipliers: [0.8, 1.0, 1.0, 0.7],
            type_weights: [6.21, 13.41, 59.10, 19.89],
        }
    }
}

/// Represents an agent in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: usize,
    /// X-coordinate of the agent's position
    pub x: u32,
    /// Y-coordinate of the agent's position
    pub y: u32,
    /// Movement speed in units per time step
    pub speed: u32,
    /// Remaining steps the agent can move in the current time step
    pub remaining_steps: u32,
    /// Whether the agent is on a road
    pub is_on_road: bool,
    /// Type of the agent
    pub agent_type: AgentType,
    /// Whether the agent is alive
    pub is_alive: bool,
}

// Legacy constant for backward compatibility
pub const BASE_SPEED: f64 = 2.66;

impl Agent {
    /// Create a new agent with default configuration
    pub fn new(id: usize, x: u32, y: u32, agent_type: AgentType, is_on_road: bool) -> Self {
        Self::with_config(id, x, y, agent_type, is_on_road, &AgentConfig::default())
    }
    
    /// Create a new agent with custom configuration
    pub fn with_config(id: usize, x: u32, y: u32, agent_type: AgentType, is_on_road: bool, config: &AgentConfig) -> Self {
        let speed = match agent_type {
            AgentType::Child => config.speed_multipliers[0] * config.base_speed,
            AgentType::Teen => config.speed_multipliers[1] * config.base_speed,
            AgentType::Adult => config.speed_multipliers[2] * config.base_speed,
            AgentType::Elder => config.speed_multipliers[3] * config.base_speed,
            AgentType::Custom(multiplier) => multiplier * config.base_speed,
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
    /// Generate a random agent type based on default distribution
    pub fn random() -> Self {
        Self::random_with_weights(&[6.21, 13.41, 59.10, 19.89])
    }
    
    /// Generate a random agent type with custom distribution weights
    pub fn random_with_weights(weights: &[f64]) -> Self {
        let variants = [
            AgentType::Child,
            AgentType::Teen,
            AgentType::Adult,
            AgentType::Elder,
        ];

        let mut rng = thread_rng();
        let dist = WeightedIndex::new(weights).unwrap();
        variants[dist.sample(&mut rng)]
    }
    
    /// Get the speed multiplier for this agent type
    pub fn speed_multiplier(&self, config: &AgentConfig) -> f64 {
        match self {
            AgentType::Child => config.speed_multipliers[0],
            AgentType::Teen => config.speed_multipliers[1],
            AgentType::Adult => config.speed_multipliers[2],
            AgentType::Elder => config.speed_multipliers[3],
            AgentType::Custom(multiplier) => *multiplier,
        }
    }
}

/// Data about dead agents for a specific step
#[derive(Serialize, Deserialize)]
pub struct DeadAgentsData {
    pub step: u32,
    pub dead_agents: usize,
}