#!/bin/bash

# First build the project
cargo build

# Run the API server with our sample data
# We'll use curl to update the configuration once the server is running
cargo run --bin api_server &

# Store the server PID
SERVER_PID=$!

# Wait for the server to start
sleep 2

# Configure the server to use our sample data
curl -X POST http://localhost:8080/config -H "Content-Type: application/json" -d '{
  "location": "sample",
  "grid_path": "./data_sample/sample_grid.asc",
  "population_path": "./data_sample/sample_agents.asc",
  "tsunami_data_path": "./data_sample/tsunami_ascii_sample",
  "output_path": "./output",
  "max_steps": 100
}'

echo -e "\n\nTsunami Simulation API server is running with sample data."
echo "To test the API, try these commands:"
echo "  curl -X POST http://localhost:8080/init"
echo "  curl -X POST http://localhost:8080/step"
echo "  curl -X GET http://localhost:8080/status"
echo ""
echo "To stop the server, press Ctrl+C"

# Wait for Ctrl+C
wait $SERVER_PID
