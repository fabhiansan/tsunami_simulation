#!/usr/bin/env python3
"""
Generate sample data for tsunami simulation testing.
This script creates:
1. A grid file with terrain data (roads, shelters, and agents)
2. Tsunami frames for simulation
"""

import os
import numpy as np

# Ensure the output directory exists
DATA_DIR = "./data_sample"
os.makedirs(DATA_DIR, exist_ok=True)

def create_sample_grid(size=20, output_path="./data_sample/sample_grid.asc"):
    """Create a sample grid file with roads, shelters, and agents."""
    # Grid metadata
    ncols = size
    nrows = size
    xllcorner = 0.0
    yllcorner = 0.0
    cellsize = 10.0
    nodata_value = -9999
    
    # Create a grid array filled with blocked terrain (0)
    terrain_grid = np.zeros((nrows, ncols), dtype=int)
    
    # Add horizontal road in the middle (value 1)
    terrain_grid[nrows // 2, :] = 1
    
    # Add vertical road in the middle
    terrain_grid[:, ncols // 2] = 1
    
    # Add another horizontal road
    terrain_grid[nrows // 4, :] = 1
    
    # Add another vertical road
    terrain_grid[:, ncols // 4] = 1
    
    # Add shelters (value 201 and 202)
    # Shelter 1 at top right
    terrain_grid[2, ncols-3] = 201
    # Shelter 2 at bottom left
    terrain_grid[nrows-3, 2] = 202
    
    # Find all road cells (value 1)
    road_cells = np.where(terrain_grid == 1)
    road_positions = list(zip(road_cells[0], road_cells[1]))
    
    # Choose random positions for agents along roads
    np.random.seed(42)  # For reproducibility
    num_agents = min(20, len(road_positions))
    agent_positions = np.random.choice(len(road_positions), num_agents, replace=False)
    
    # Assign agent types
    agent_types = {
        3: "Adult",
        4: "Child",
        5: "Teen",
        6: "Elder"
    }
    
    # Distribute agent types
    for i, pos_idx in enumerate(agent_positions):
        row, col = road_positions[pos_idx]
        
        # Determine agent type (cycle through types)
        agent_type = 3 + (i % 4)  # 3, 4, 5, 6
        
        # Place agent on grid
        terrain_grid[row, col] = agent_type
        
        print(f"Placed {agent_types[agent_type]} agent at position ({row}, {col})")
    
    # Write the grid to an ASCII file
    with open(output_path, 'w', encoding='utf-8') as f:
        # Write header
        f.write(f"ncols {ncols}\n")
        f.write(f"nrows {nrows}\n")
        f.write(f"xllcorner {xllcorner}\n")
        f.write(f"yllcorner {yllcorner}\n")
        f.write(f"cellsize {cellsize}\n")
        f.write(f"NODATA_value {nodata_value}\n")
        
        # Write grid data
        for row in terrain_grid:
            f.write(' '.join(map(str, row)) + '\n')
    
    print(f"Created sample grid with agents at {output_path}")
    return terrain_grid

def create_mock_tsunami_data(grid, num_frames=5, output_dir="./data_sample/tsunami_ascii_sample"):
    """Create mock tsunami data frames."""
    nrows, ncols = grid.shape
    os.makedirs(output_dir, exist_ok=True)
    
    # Create tsunami frames (simple wave from left to right)
    for frame in range(num_frames):
        tsunami_grid = np.zeros((nrows, ncols), dtype=int)
        
        # Create a simple advancing wave
        wave_position = int((frame / num_frames) * ncols)
        wave_width = max(3, int(ncols / 10))
        
        # Create the wave
        for col in range(max(0, wave_position), min(ncols, wave_position + wave_width)):
            for row in range(nrows):
                # Higher tsunami height near the coast (bottom of grid)
                height = int(10 * (nrows - row) / nrows)
                tsunami_grid[row, col] = height
        
        # Write the tsunami frame to an ASCII file
        output_path = os.path.join(output_dir, f"tsunami_{frame:03d}.asc")
        with open(output_path, 'w', encoding='utf-8') as f:
            # Write header
            f.write(f"ncols {ncols}\n")
            f.write(f"nrows {nrows}\n")
            f.write(f"xllcorner {0.0}\n")
            f.write(f"yllcorner {0.0}\n")
            f.write(f"cellsize {10.0}\n")
            f.write(f"NODATA_value {-9999}\n")
            
            # Write grid data
            for row in tsunami_grid:
                f.write(' '.join(map(str, row)) + '\n')
        
        print(f"Created tsunami frame {frame} at {output_path}")

if __name__ == "__main__":
    # Generate sample grid with agents
    grid = create_sample_grid()
    
    # Generate mock tsunami data
    create_mock_tsunami_data(grid)
    
    print("Sample data generation complete!")
