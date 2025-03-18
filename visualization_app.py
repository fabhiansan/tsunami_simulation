#!/usr/bin/env python3
"""
Visualization App for Tsunami Simulation

This Flask application provides a web interface to visualize the tsunami simulation,
with controls to step forward and backward through the simulation.
"""

import os
import json
import requests
from flask import Flask, render_template, jsonify, request, send_from_directory
from flask_cors import CORS

app = Flask(__name__)
CORS(app)

# Configuration
API_BASE_URL = "http://localhost:8080"  # Rust API server
TEMPLATES_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "templates")
STATIC_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "static")

# Create directories if they don't exist
os.makedirs(TEMPLATES_DIR, exist_ok=True)
os.makedirs(STATIC_DIR, exist_ok=True)
os.makedirs(os.path.join(STATIC_DIR, "css"), exist_ok=True)
os.makedirs(os.path.join(STATIC_DIR, "js"), exist_ok=True)

# Store simulation history for backward stepping
simulation_history = []

# Routes
@app.route('/')
def index():
    """Render the main visualization page"""
    return render_template('visualization.html')

@app.route('/simple')
def simple_view():
    """Render a simplified visualization page for troubleshooting"""
    return render_template('simple_visualization.html')

@app.route('/static/<path:path>')
def serve_static(path):
    """Serve static files"""
    return send_from_directory(STATIC_DIR, path)

@app.route('/api/init', methods=['POST'])
def init_simulation():
    """Initialize the simulation via the Rust API"""
    try:
        response = requests.post(f"{API_BASE_URL}/init")
        data = response.json()
        
        # Reset history when initializing
        global simulation_history
        simulation_history = []
        
        return jsonify(data)
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

@app.route('/api/status')
def get_status():
    """Get current simulation status"""
    try:
        response = requests.get(f"{API_BASE_URL}/status")
        return jsonify(response.json())
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

@app.route('/api/config')
def get_config():
    """Get simulation configuration"""
    try:
        response = requests.get(f"{API_BASE_URL}/config")
        return jsonify(response.json())
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

@app.route('/api/grid')
def get_grid():
    """Get grid data from the api"""
    try:
        # Forward the request to the Rust API's grid endpoint
        response = requests.get(f"{API_BASE_URL}/grid", timeout=5)
        
        if response.status_code != 200:
            # If simulation isn't initialized, we need to initialize it first
            if response.status_code == 400:
                # Try to initialize the simulation
                init_response = requests.post(f"{API_BASE_URL}/init", timeout=10)
                
                if init_response.status_code != 200:
                    return jsonify({
                        "status": "error",
                        "message": f"Failed to initialize simulation: {init_response.text}"
                    }), init_response.status_code
                
                # Try to get grid data again after initialization
                response = requests.get(f"{API_BASE_URL}/grid", timeout=5)
                
                if response.status_code != 200:
                    return jsonify({
                        "status": "error",
                        "message": f"Failed to fetch grid data after initialization: {response.text}"
                    }), response.status_code
            else:
                return jsonify({
                    "status": "error",
                    "message": f"Failed to fetch grid data: {response.text}"
                }), response.status_code
        
        # Return the response from the Rust API
        return response.json()
        
    except requests.RequestException as e:
        return jsonify({
            "status": "error",
            "message": f"Error connecting to API server: {str(e)}"
        }), 500
    except Exception as e:
        return jsonify({
            "status": "error",
            "message": f"Unexpected error: {str(e)}"
        }), 500

@app.route('/api/step/forward', methods=['POST'])
def step_forward():
    """Run one simulation step forward and save the state in history"""
    try:
        print("Step forward requested")
        
        # Get the current state first
        print("Fetching current state from API...")
        state_response = requests.get(f"{API_BASE_URL}/status", timeout=5)
        print(f"State response status: {state_response.status_code}")
        current_state = state_response.json() if state_response.status_code == 200 else None
        print(f"Current state: {current_state}")
        
        # If simulation is not initialized, try to initialize it first
        if not current_state or not current_state.get('state', {}).get('is_running', False):
            try:
                print("Simulation not initialized, attempting to initialize...")
                # Try to initialize the simulation
                init_response = requests.post(f"{API_BASE_URL}/init", timeout=5)
                print(f"Init response status: {init_response.status_code}")
                if init_response.status_code != 200:
                    # If initialization fails, let's create a basic state for simulation
                    print("Failed to initialize, continuing anyway")
                else:
                    print("Initialization successful")
            except Exception as e:
                print(f"Error initializing: {e}")
        
        # Get current step number
        current_step = 0
        if current_state and 'state' in current_state:
            current_step = current_state['state'].get('current_step', 0)
        print(f"Current step: {current_step}")
        
        # Save current state to history if available
        if current_state:
            global simulation_history
            print(f"Current history length: {len(simulation_history)}")
            
            # Trim history to current step if we're not at the end
            if len(simulation_history) > current_step:
                print(f"Trimming history from {len(simulation_history)} to {current_step + 1}")
                simulation_history = simulation_history[:current_step + 1]
            
            # Add current state to history if not already there
            if current_step >= len(simulation_history):
                print(f"Adding current state to history at index {current_step}")
                simulation_history.append(current_state)
        
        # Step forward
        print("Sending step request to API...")
        step_response = requests.post(f"{API_BASE_URL}/step", timeout=5)
        print(f"Step response status: {step_response.status_code}")
        
        if step_response.status_code != 200:
            error_msg = f"Failed to step simulation: {step_response.text}"
            print(error_msg)
            return jsonify({"status": "error", "message": error_msg})
        
        step_data = step_response.json()
        print(f"Step data: {step_data}")
        
        # Get updated state after stepping
        print("Fetching updated state after step...")
        new_state_response = requests.get(f"{API_BASE_URL}/status", timeout=5)
        print(f"New state response status: {new_state_response.status_code}")
        new_state = new_state_response.json() if new_state_response.status_code == 200 else None
        print(f"New state: {new_state}")
        
        # Add new state to history
        if new_state:
            new_step = new_state['state'].get('current_step', current_step + 1)
            print(f"New step: {new_step}")
            
            if len(simulation_history) <= new_step:
                print(f"Adding new state to history at index {new_step}")
                simulation_history.append(new_state)
        
        result = {
            "status": "ok", 
            "current_state": new_state,
            "current_step": new_state['state'].get('current_step', current_step + 1) if new_state else current_step + 1
        }
        print(f"Returning result: {result}")
        return jsonify(result)
    except Exception as e:
        error_msg = f"Error in step_forward: {str(e)}"
        print(error_msg)
        return jsonify({"status": "error", "message": error_msg})

@app.route('/api/step/backward', methods=['POST'])
def step_backward():
    """Go one step backward in the simulation using saved history"""
    global simulation_history
    
    try:
        if not simulation_history:
            return jsonify({"status": "error", "message": "No history available"})
        
        # Get current state
        current_state_resp = requests.get(f"{API_BASE_URL}/status")
        current_step = current_state_resp.json()['state']['current_step']
        
        # Reset the simulation
        requests.post(f"{API_BASE_URL}/reset")
        requests.post(f"{API_BASE_URL}/init")
        
        # Replay the simulation up to the previous step
        target_step = current_step - 2 if current_step > 1 else 0
        
        for i in range(target_step + 1):
            if i < len(simulation_history):
                requests.post(f"{API_BASE_URL}/step")
        
        # Get updated state
        updated_state_resp = requests.get(f"{API_BASE_URL}/status")
        updated_state = updated_state_resp.json()
        
        # Remove the last history entry
        if simulation_history and simulation_history[-1]['step'] >= target_step:
            simulation_history.pop()
        
        return jsonify({
            "status": "ok",
            "previous_step": current_step,
            "current_step": updated_state['state']['current_step'],
            "current_state": updated_state
        })
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

@app.route('/api/reset', methods=['POST'])
def reset_simulation():
    """Reset the simulation"""
    try:
        response = requests.post(f"{API_BASE_URL}/reset")
        
        # Clear history
        global simulation_history
        simulation_history = []
        
        return jsonify(response.json())
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

@app.route('/api/history')
def get_history():
    """Get the simulation history"""
    return jsonify({"history": simulation_history})

@app.route('/api/export')
def export_results():
    """Proxy requests to the Rust API's /export endpoint to get agent positions and states"""
    try:
        # Forward the request directly to the Rust API without checking status
        export_response = requests.get(f"{API_BASE_URL}/export", timeout=5)
        
        if export_response.status_code != 200:
            return jsonify({
                "status": "error",
                "message": f"Failed to fetch export data: {export_response.status_code}"
            }), export_response.status_code
            
        # Return the response from the Rust API
        return export_response.json()
        
    except requests.RequestException as e:
        return jsonify({
            "status": "error",
            "message": f"Error connecting to API server: {str(e)}"
        }), 500
    except Exception as e:
        return jsonify({
            "status": "error",
            "message": f"Unexpected error: {str(e)}"
        }), 500

if __name__ == '__main__':
    app.run(debug=True, host='127.0.0.1', port=5001)
