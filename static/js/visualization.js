// Tsunami Simulation Visualization - Main JavaScript

// Global variables
let gridData = null;
let gridCanvas = null;
let gridContext = null;
let cellSize = 30; // Size of each cell in pixels
let gridLoaded = false;
let simulationInitialized = false;
let currentState = null;
let historyIndex = 0;

// Color mapping for different cell types
const cellColors = {
    0: "#f8f9fa",    // Blocked terrain
    1: "#adb5bd",    // Custom terrain
    2: "#6c757d",    // Road
    3: "#198754",    // Shelter
    // Agent types (will be drawn separately)
    "Adult": "#0d6efd",
    "Child": "#0dcaf0",
    "Teen": "#ffc107",
    "Elder": "#dc3545"
};

// Wait for DOM to be fully loaded
document.addEventListener('DOMContentLoaded', function() {
    console.log("DOM fully loaded");
    
    // Get canvas element
    gridCanvas = document.getElementById('grid-canvas');
    gridContext = gridCanvas.getContext('2d');
    
    // Set up event listeners for buttons
    const initBtn = document.getElementById('init-btn');
    const stepForwardBtn = document.getElementById('step-forward-btn');
    const stepBackwardBtn = document.getElementById('step-backward-btn');
    const resetBtn = document.getElementById('reset-btn');
    
    console.log("Step Forward button found:", stepForwardBtn !== null);
    
    initBtn.addEventListener('click', initializeSimulation);
    console.log("Added click listener to Initialize button");
    
    stepForwardBtn.addEventListener('click', function(e) {
        console.log("Step Forward button clicked!");
        e.preventDefault();
        stepForward();
    });
    console.log("Added click listener to Step Forward button");
    
    stepBackwardBtn.addEventListener('click', stepBackward);
    resetBtn.addEventListener('click', resetSimulation);
    
    // Load grid data initially
    loadGridData();
    
    // Check simulation status
    updateStatus();
    
    // Set up automatic status updates
    setInterval(updateStatus, 2000);
    
    // Explicitly enable the Step Forward button regardless of initialization state
    stepForwardBtn.disabled = false;
    console.log("Explicitly enabled Step Forward button, disabled status:", stepForwardBtn.disabled);
    
    // Log initialization
    logMessage("Visualization interface loaded. Please initialize the simulation.", "info");
});

// Function to load grid data for visualization
async function loadGridData() {
    try {
        // Log attempt
        logMessage("Loading grid data...", "info");
        console.log("Fetching grid data");
        
        // Fetch grid data
        const response = await fetch('/api/grid');
        
        if (!response.ok) {
            throw new Error(`Failed to load grid data: ${response.status}`);
        }
        
        const data = await response.json();
        gridData = data;
        
        console.log("Grid data received:", data);
        
        // Extract grid dimensions
        const nrows = data.grid.length;
        const ncols = data.grid[0].length;
        
        // Set canvas size based on grid dimensions
        setCanvasSize(nrows, ncols);
        
        // Resize canvas
        gridCanvas.width = ncols * cellSize;
        gridCanvas.height = nrows * cellSize;
        
        // Draw initial grid
        drawGrid(data.grid);
        gridLoaded = true;
        
        // Enable step forward button if grid is loaded, even without initialization
        document.getElementById('step-forward-btn').disabled = false;
        
        // Update with real-time agent positions if available
        await updateAgentPositions();
        
        logMessage(`Grid data loaded successfully: ${nrows}x${ncols} grid with cell size ${cellSize}px`, "success");
    } catch (error) {
        console.error("Error loading grid data:", error);
        logMessage(`Error loading grid data: ${error.message}`, "danger");
    }
}

// Function to draw the grid
function drawGrid(grid) {
    if (!gridContext) {
        console.error("No grid context available for drawing");
        logMessage("Canvas context not available for drawing grid", "danger");
        return;
    }
    
    console.log("Drawing grid...");
    
    // Clear canvas
    gridContext.clearRect(0, 0, gridCanvas.width, gridCanvas.height);
    
    const nrows = grid.length;
    const ncols = grid[0].length;
    
    console.log(`Drawing grid with ${nrows} rows and ${ncols} columns`);
    
    // Draw each cell
    for (let row = 0; row < nrows; row++) {
        for (let col = 0; col < ncols; col++) {
            const cellType = grid[row][col];
            const x = col * cellSize;
            const y = row * cellSize;
            
            // Set fill color based on cell type
            gridContext.fillStyle = getCellColor(cellType);
            
            // Draw filled rectangle for the cell
            gridContext.fillRect(x, y, cellSize, cellSize);
            
            // Draw border
            gridContext.strokeStyle = "#dee2e6";
            gridContext.strokeRect(x, y, cellSize, cellSize);
            
            // Only add text labels for special terrain (not for agents anymore)
            if (cellType > 1 && cellType < 200) {
                gridContext.fillStyle = getTextColor(cellType);
                gridContext.font = '10px Arial';
                gridContext.textAlign = 'center';
                gridContext.textBaseline = 'middle';
                gridContext.fillText(cellType.toString(), x + cellSize/2, y + cellSize/2);
            }
        }
    }
}

// Function to draw agents on top of the grid
function drawAgents(agents, isTsunami, tsunamiCells) {
    if (!gridContext || !gridData) {
        console.error("No grid context or data available for drawing agents");
        return;
    }
    
    console.log(`Drawing ${agents.length} agents...`);
    
    // Draw tsunami cells first if tsunami is active
    if (isTsunami && tsunamiCells && tsunamiCells.length > 0) {
        console.log(`Drawing tsunami with ${tsunamiCells.length} active cells`);
        
        // Add a semi-transparent blue overlay for tsunami cells
        gridContext.globalAlpha = 0.5;
        
        for (const cell of tsunamiCells) {
            const x = cell.x * cellSize;
            const y = cell.y * cellSize;
            
            // Different blue shades based on tsunami height
            const blueIntensity = Math.min(255, 100 + (cell.height * 5));
            gridContext.fillStyle = `rgba(0, 0, ${blueIntensity}, 0.7)`;
            
            // Draw filled rectangle for tsunami
            gridContext.fillRect(x, y, cellSize, cellSize);
        }
        
        // Reset alpha
        gridContext.globalAlpha = 1.0;
    }
    
    // Draw each agent
    for (const agent of agents) {
        // Skip agents that aren't alive
        if (!agent.is_alive) continue;
        
        const x = agent.x * cellSize;
        const y = agent.y * cellSize;
        
        // Draw agent icon based on type
        const agentRadius = cellSize / 3;
        
        // Circle for the agent
        gridContext.beginPath();
        gridContext.arc(x + cellSize/2, y + cellSize/2, agentRadius, 0, 2 * Math.PI);
        
        // Color based on agent type (now a string from the API)
        const agentColor = getAgentColor(agent.type);
        gridContext.fillStyle = agentColor;
        gridContext.fill();
        
        // Add a border
        gridContext.strokeStyle = "#000000";
        gridContext.lineWidth = 1;
        gridContext.stroke();
    }
}

// Function to get agent color based on agent type
function getAgentColor(agentType) {
    // Agent types are now strings from the API
    if (cellColors.hasOwnProperty(agentType)) {
        return cellColors[agentType];
    }
    
    // Fallback colors based on type
    switch (agentType) {
        case "Adult":
            return "#0d6efd"; // Blue
        case "Child":
            return "#0dcaf0"; // Cyan
        case "Teen":
            return "#ffc107"; // Yellow
        case "Elder":
            return "#dc3545"; // Red
        default:
            return "#6c757d"; // Gray as default
    }
}

// Function to get cell color based on type
function getCellColor(cellType) {
    // If the cell type is in our map, return that color
    if (cellColors.hasOwnProperty(cellType)) {
        return cellColors[cellType];
    }
    
    // Handle shelter IDs (> 200)
    if (cellType >= 200 && cellType < 300) {
        return "#198754"; // Green for shelters
    }
    
    // Default color for unknown types
    return "#f8f9fa";
}

// Function to get text color for labels
function getTextColor(cellType) {
    // Dark background cells need white text
    if (cellType === 3 || cellType === 4 || cellType === 6 || (cellType >= 200 && cellType < 300)) {
        return "#ffffff";
    }
    
    // Light background cells need dark text
    return "#000000";
}

// Function to fetch and display real-time agent data
async function updateAgentPositions() {
    try {
        console.log("Updating agent positions...");
        
        // Fetch agent data
        const response = await fetch('/api/export');
        
        if (!response.ok) {
            throw new Error(`Failed to get agent data: ${response.status}`);
        }
        
        const data = await response.json();
        
        if (data.status !== "ok") {
            throw new Error(`API returned error: ${data.message || "Unknown error"}`);
        }
        
        // Check if we have a valid grid to draw on
        if (!gridData || !gridContext) {
            console.error("No grid data or context available");
            return;
        }
        
        // Redraw the base grid first to clear old agent positions
        if (gridData.grid) {
            drawGrid(gridData.grid);
        }
        
        // Draw agents if we have them
        if (data.agents && Array.isArray(data.agents)) {
            // Draw the agents
            drawAgents(data.agents, data.is_tsunami, data.tsunami_cells);
            
            // Update UI with counts - use the safe update method
            updateUIElement('dead-agents', data.dead_agents || 0);
            updateUIElement('is-tsunami', data.is_tsunami ? "Yes" : "No");
            updateUIElement('current-step', data.step || 0);
            updateUIElement('total-agents', data.total_agents || 0);
            
            logMessage(`Updated ${data.agents.length} agent positions at step ${data.step}`, "info");
        } else {
            console.warn("No agent data available in the response", data);
            logMessage("No agent data available from API", "warning");
        }
    } catch (error) {
        console.error("Error updating agent positions:", error);
        logMessage(`Error updating agent positions: ${error.message}`, "danger");
    }
}

// Function to initialize the simulation
async function initializeSimulation() {
    try {
        // Disable buttons during initialization
        setButtonStates(true, true, true, true);
        
        // Log attempt
        logMessage("Initializing simulation...", "info");
        console.log("Initializing simulation");
        
        // Initialize simulation
        const response = await fetch('/api/init', {
            method: 'POST'
        });
        
        if (!response.ok) {
            throw new Error(`Failed to initialize simulation: ${response.status}`);
        }
        
        const data = await response.json();
        console.log("Initialization data:", data);
        
        if (data.status === "error") {
            throw new Error(`API returned error: ${data.message}`);
        }
        
        // Load initial grid data
        await loadGridData();
        
        // Update with real-time agent positions
        await updateAgentPositions();
        
        // Update status
        await updateStatus();
        
        // Enable appropriate buttons
        setButtonStates(false, false, true, false);
        
        logMessage("Simulation initialized successfully", "success");
    } catch (error) {
        console.error("Error initializing simulation:", error);
        logMessage(`Error initializing simulation: ${error.message}`, "danger");
        
        // Even if initialization fails, try to load grid data
        try {
            await loadGridData();
        } catch (gridError) {
            console.error("Failed to load grid after initialization error:", gridError);
        }
        
        // Re-enable buttons except step backward
        setButtonStates(false, false, true, false);
    }
}

// Function to step forward in the simulation
async function stepForward() {
    try {
        console.log("Attempting to step forward in simulation...");
        logMessage("Attempting to step forward in simulation...", "info");
        
        // Disable step button during request
        const stepBtn = document.getElementById('step-forward-btn');
        if (stepBtn) stepBtn.disabled = true;
        
        // Call the step forward API endpoint
        const response = await fetch('/api/step/forward', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        if (!response.ok) {
            throw new Error(`Failed to step forward: ${response.status}`);
        }
        
        const data = await response.json();
        
        if (data.status === "error") {
            throw new Error(`API returned error: ${data.message}`);
        }
        
        // Update simulation stats 
        updateUIElement('current-step', data.current_step);
        
        // Update agent positions
        await updateAgentPositions();
        
        logMessage(`Stepped forward to step ${data.current_step}`, "success");
    } catch (error) {
        console.error("Error stepping forward:", error);
        logMessage(`Error stepping forward: ${error.message}`, "danger");
    } finally {
        // Re-enable button
        const stepBtn = document.getElementById('step-forward-btn');
        if (stepBtn) stepBtn.disabled = false;
    }
}

// Function to step backward in the simulation
async function stepBackward() {
    try {
        // Disable buttons during step
        setButtonStates(true, true, true, true);
        
        // Step simulation backward
        const response = await fetch('/api/step/backward', {
            method: 'POST'
        });
        
        if (!response.ok) {
            throw new Error(`Failed to step backward: ${response.status}`);
        }
        
        const data = await response.json();
        
        // Check if operation was successful
        if (data.status === "ok") {
            // Reload grid data to see changes
            await loadGridData();
            
            // Update status display
            updateStatus();
            
            // Enable appropriate buttons - disable backward if at step 0
            const disableBackward = data.current_step <= 0;
            setButtonStates(false, false, disableBackward, false);
            
            historyIndex--;
            
            logMessage(`Went back to step ${data.current_step}.`, "info");
        } else {
            throw new Error(data.message || "Failed to step backward");
        }
    } catch (error) {
        logMessage(`Error stepping backward: ${error.message}`, "danger");
        setButtonStates(false, false, true, false);
    }
}

// Function to reset the simulation
async function resetSimulation() {
    try {
        // Disable buttons during reset
        setButtonStates(true, true, true, true);
        
        // Reset simulation
        const response = await fetch('/api/reset', {
            method: 'POST'
        });
        
        if (!response.ok) {
            throw new Error(`Failed to reset simulation: ${response.status}`);
        }
        
        const data = await response.json();
        
        if (data.status === "ok") {
            simulationInitialized = false;
            historyIndex = 0;
            
            // Reload grid data
            await loadGridData();
            
            // Update status display
            updateStatus();
            
            // Set appropriate button states
            setButtonStates(false, true, true, false);
            
            logMessage("Simulation reset successfully.", "warning");
        } else {
            throw new Error(data.message || "Failed to reset simulation");
        }
    } catch (error) {
        logMessage(`Error resetting simulation: ${error.message}`, "danger");
        setButtonStates(false, true, true, false);
    }
}

// Function to update simulation status display
async function updateStatus() {
    try {
        // Get current status
        const response = await fetch('/api/status');
        
        if (!response.ok) {
            throw new Error(`Failed to get status: ${response.status}`);
        }
        
        const data = await response.json();
        
        // Update UI elements safely
        updateUIElement('current-step', data.state.current_step);
        updateUIElement('total-agents', data.total_agents);
        updateUIElement('agents-in-shelters', data.agents_in_shelters);
        updateUIElement('dead-agents', data.state.dead_agents);
        updateUIElement('is-tsunami', data.state.is_tsunami ? "Yes" : "No");
        
        // Update button states based on simulation status
        simulationInitialized = data.state.is_running;
        setButtonStates(!simulationInitialized, false, historyIndex <= 0, false);
        
    } catch (error) {
        console.error("Error updating status:", error);
        // Don't show error message for status updates to avoid cluttering the UI
    }
}

// Helper function to set button states
function setButtonStates(initDisabled, stepForwardDisabled, stepBackwardDisabled, resetDisabled) {
    document.getElementById('init-btn').disabled = initDisabled;
    document.getElementById('step-forward-btn').disabled = stepForwardDisabled;
    document.getElementById('step-backward-btn').disabled = stepBackwardDisabled;
    document.getElementById('reset-btn').disabled = resetDisabled;
}

// Helper function to safely update UI elements
function updateUIElement(elementId, value) {
    const element = document.getElementById(elementId);
    if (element) {
        element.textContent = value;
    } else {
        console.warn(`Element ${elementId} not found in DOM`);
    }
}

// Helper function to log messages in the UI
function logMessage(message, level = "info") {
    const logContainer = document.getElementById('log-container');
    const now = new Date();
    const timeString = now.toLocaleTimeString();
    
    const logEntry = document.createElement('div');
    logEntry.classList.add('log-entry', `log-${level}`);
    logEntry.innerHTML = `<span class="log-time">[${timeString}]</span> ${message}`;
    
    // Add to top of log
    logContainer.prepend(logEntry);
}

// Function to set canvas size and adjust cell size if needed based on grid dimensions
function setCanvasSize(nrows, ncols) {
    const maxCanvasWidth = 1200;
    const maxCanvasHeight = 800;
    const minCellSize = 4;
    
    // Start with default cell size if not already set
    if (!cellSize) {
        cellSize = 10;
    }
    
    // Calculate required dimensions
    let requiredWidth = ncols * cellSize;
    let requiredHeight = nrows * cellSize;
    
    // If dimensions exceed max, scale down cell size while maintaining minimum
    if (requiredWidth > maxCanvasWidth || requiredHeight > maxCanvasHeight) {
        const widthRatio = maxCanvasWidth / requiredWidth;
        const heightRatio = maxCanvasHeight / requiredHeight;
        
        // Use the smaller ratio to ensure both dimensions fit
        const ratio = Math.min(widthRatio, heightRatio);
        
        // Scale down cellSize, but not below minimum
        cellSize = Math.max(minCellSize, Math.floor(cellSize * ratio));
        
        console.log(`Adjusting cell size to ${cellSize}px to fit canvas constraints`);
    }
    
    console.log(`Canvas will be ${ncols * cellSize}x${nrows * cellSize} with cell size ${cellSize}px`);
}
