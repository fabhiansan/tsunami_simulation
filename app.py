from flask import Flask, render_template, jsonify
from flask_cors import CORS
import os
import json
import logging
import numpy as np
from collections import defaultdict
import time
import ijson  # Add ijson for streaming JSON parsing
from decimal import Decimal

# Konfigurasi logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)
CORS(app)

# Direktori tempat hasil simulasi disimpan
OUTPUT_DIR = 'output'
# Cache untuk menyimpan data
processed_data = None
metadata_cache = None

def process_geojson_streaming():
    """Process GeoJSON file using streaming parser to handle large files efficiently"""
    global processed_data, metadata_cache
    
    if processed_data is None:
        start_time = time.time()
        geojson_path = os.path.join(OUTPUT_DIR, 'custom_simulation.geojson')
        logger.info(f"Processing GeoJSON file from: {geojson_path}")
        
        # Initialize data structure
        timesteps = defaultdict(lambda: {'x': [], 'y': [], 'types': []})
        invalid_coords_count = 0
        valid_coords_count = 0
        
        # Track reasons for invalid coordinates
        invalid_reasons = {
            "missing_geometry": 0,
            "empty_coords": 0,
            "non_numeric": 0,
            "non_finite": 0,
            "index_error": 0,
            "type_error": 0,
            "value_error": 0,
            "other": 0
        }
        
        # Debug: Sample some invalid coordinates for debugging
        sample_invalid_coords = []
        
        # Process file in streaming mode
        with open(geojson_path, 'r') as f:
            # Parse features one by one
            for feature_idx, feature in enumerate(ijson.items(f, 'features.item')):
                # Print debug info for first few features
                if feature_idx < 5:
                    logger.info(f"Feature {feature_idx}: {feature.keys()}")
                    if 'geometry' in feature:
                        logger.info(f"  Geometry type: {feature['geometry'].get('type')}")
                        if 'coordinates' in feature['geometry']:
                            logger.info(f"  Coordinates sample: {feature['geometry']['coordinates'][:2]}")
                
                time_step = feature.get('properties', {}).get('timestamp', 0)
                agent_type = feature.get('properties', {}).get('agent_type', 'Unknown')
                
                # Check if the geometry is present and has coordinates
                if 'geometry' not in feature or 'coordinates' not in feature['geometry']:
                    invalid_reasons["missing_geometry"] += 1
                    invalid_coords_count += 1
                    continue
                
                coords = feature['geometry']['coordinates']
                geometry_type = feature['geometry'].get('type', '')
                
                # Handle different geometry types
                if geometry_type == 'MultiPoint':
                    # For MultiPoint, we have an array of coordinate pairs
                    # Process each coordinate pair in the MultiPoint
                    for coord_idx, coord_pair in enumerate(coords):
                        try:
                            # Validate coordinates
                            if len(coord_pair) < 2:
                                invalid_reasons["empty_coords"] += 1
                                invalid_coords_count += 1
                                continue
                                
                            # Convert Decimal to float if needed
                            x = float(coord_pair[0]) if isinstance(coord_pair[0], Decimal) else coord_pair[0]
                            y = float(coord_pair[1]) if isinstance(coord_pair[1], Decimal) else coord_pair[1]
                            
                            # Skip invalid coordinates
                            if x is None or y is None:
                                invalid_reasons["empty_coords"] += 1
                                invalid_coords_count += 1
                                continue
                                
                            if not isinstance(x, (int, float)) or not isinstance(y, (int, float)):
                                invalid_reasons["non_numeric"] += 1
                                invalid_coords_count += 1
                                # Save sample for debugging
                                if len(sample_invalid_coords) < 5:
                                    sample_invalid_coords.append((x, y, "non_numeric"))
                                continue
                                
                            if not np.isfinite(x) or not np.isfinite(y):
                                invalid_reasons["non_finite"] += 1
                                invalid_coords_count += 1
                                # Save sample for debugging
                                if len(sample_invalid_coords) < 5:
                                    sample_invalid_coords.append((x, y, "non_finite"))
                                continue
                                
                            timesteps[time_step]['x'].append(x)
                            timesteps[time_step]['y'].append(y)
                            timesteps[time_step]['types'].append(agent_type)
                            valid_coords_count += 1
                        except IndexError as e:
                            invalid_reasons["index_error"] += 1
                            invalid_coords_count += 1
                        except TypeError as e:
                            invalid_reasons["type_error"] += 1
                            invalid_coords_count += 1
                        except ValueError as e:
                            invalid_reasons["value_error"] += 1
                            invalid_coords_count += 1
                        except Exception as e:
                            invalid_reasons["other"] += 1
                            invalid_coords_count += 1
                else:
                    # Handle Point or other geometry types
                    try:
                        # For Point geometry, coords is a single coordinate pair
                        # For other geometries, try to extract the first coordinate
                        if isinstance(coords[0], list):
                            x_raw = coords[0][0]
                            y_raw = coords[0][1]
                        else:
                            x_raw = coords[0]
                            y_raw = coords[1]
                        
                        # Convert Decimal to float if needed
                        x = float(x_raw) if isinstance(x_raw, Decimal) else x_raw
                        y = float(y_raw) if isinstance(y_raw, Decimal) else y_raw
                        
                        # Skip invalid coordinates
                        if x is None or y is None:
                            invalid_reasons["empty_coords"] += 1
                            invalid_coords_count += 1
                            continue
                            
                        if not isinstance(x, (int, float)) or not isinstance(y, (int, float)):
                            invalid_reasons["non_numeric"] += 1
                            invalid_coords_count += 1
                            continue
                            
                        if not np.isfinite(x) or not np.isfinite(y):
                            invalid_reasons["non_finite"] += 1
                            invalid_coords_count += 1
                            continue
                            
                        timesteps[time_step]['x'].append(x)
                        timesteps[time_step]['y'].append(y)
                        timesteps[time_step]['types'].append(agent_type)
                        valid_coords_count += 1
                    except IndexError as e:
                        invalid_reasons["index_error"] += 1
                        invalid_coords_count += 1
                    except TypeError as e:
                        invalid_reasons["type_error"] += 1
                        invalid_coords_count += 1
                    except ValueError as e:
                        invalid_reasons["value_error"] += 1
                        invalid_coords_count += 1
                    except Exception as e:
                        invalid_reasons["other"] += 1
                        invalid_coords_count += 1
        
        # Convert defaultdict to regular dict
        processed_data = dict(timesteps)
        
        # Cache metadata
        all_timesteps = sorted(processed_data.keys()) if processed_data else []
        
        # Log debug information
        logger.info(f"Invalid coordinate reasons: {invalid_reasons}")
        if sample_invalid_coords:
            logger.info(f"Sample invalid coordinates: {sample_invalid_coords}")
        
        if all_timesteps:
            # We have valid timesteps
            metadata_cache = {
                'max_timestamp': max(all_timesteps),
                'min_timestamp': min(all_timesteps),
                'all_timesteps': all_timesteps,
                'total_agents': len(processed_data[all_timesteps[0]]['x']) if all_timesteps else 0,
                'valid_coords': valid_coords_count,
                'invalid_coords': invalid_coords_count,
                'invalid_reasons': invalid_reasons
            }
        else:
            # No valid timesteps found, but still create metadata
            metadata_cache = {
                'error': 'No valid timesteps found in the GeoJSON file',
                'suggestion': 'Check the GeoJSON file format and coordinate values',
                'max_timestamp': 0,
                'min_timestamp': 0,
                'all_timesteps': [],
                'total_agents': 0,
                'valid_coords': valid_coords_count,
                'invalid_coords': invalid_coords_count,
                'invalid_reasons': invalid_reasons
            }
        
        logger.info(f"Processed {len(timesteps)} timesteps in {time.time() - start_time:.2f} seconds")
        logger.info(f"Valid coordinates: {valid_coords_count}, Invalid coordinates: {invalid_coords_count}")
        
        if len(all_timesteps) == 0:
            logger.warning("No valid timesteps found in the GeoJSON file")
    
    return processed_data

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/data/metadata')
def get_metadata():
    try:
        # Process data if not already processed
        if metadata_cache is None:
            try:
                process_geojson_streaming()
            except Exception as e:
                logger.error(f"Error processing GeoJSON: {str(e)}")
                return jsonify({
                    "error": f"Error processing GeoJSON: {str(e)}",
                    "suggestion": "Check if the GeoJSON file is valid and accessible",
                    "max_timestamp": 0,
                    "min_timestamp": 0,
                    "all_timesteps": [],
                    "total_agents": 0,
                    "valid_coords": 0,
                    "invalid_coords": 0
                }), 200  # Return 200 instead of 500 so frontend can still use default values
        
        # Always return metadata, even if empty
        if metadata_cache is None:
            # Create default metadata if none exists
            default_metadata = {
                "error": "No metadata available",
                "suggestion": "Check if the GeoJSON file contains valid data",
                "max_timestamp": 0,
                "min_timestamp": 0,
                "all_timesteps": [],
                "total_agents": 0,
                "valid_coords": 0,
                "invalid_coords": 0
            }
            logger.info(f"Returning default metadata")
            return jsonify(default_metadata), 200
            
        logger.info(f"Metadata: {metadata_cache}")
        return jsonify(metadata_cache), 200
    except Exception as e:
        logger.error(f"Error in get_metadata: {str(e)}")
        return jsonify({
            "error": str(e),
            "max_timestamp": 0,
            "min_timestamp": 0,
            "all_timesteps": [],
            "total_agents": 0,
            "valid_coords": 0,
            "invalid_coords": 0
        }), 200  # Return 200 so frontend can use the default values

@app.route('/data/timestep/<int:step>')
def get_timestep_data(step):
    try:
        data = process_geojson_streaming()
        
        # Convert step to integer to ensure proper comparison
        step = int(step)
        
        if step not in data:
            available_steps = sorted(data.keys())
            logger.warning(f"Timestep {step} not found. Available steps: {available_steps[:10]}...")
            return jsonify({
                "error": f"Timestep {step} not found",
                "available_steps": available_steps[:20]  # Send first 20 available steps
            }), 404

        timestep_data = data[step]
        return jsonify(timestep_data)
        
    except Exception as e:
        logger.error(f"Error in get_timestep_data: {str(e)}")
        return jsonify({"error": str(e)}), 500

if __name__ == '__main__':
    app.run(debug=True, host='0.0.0.0', port=5001) 