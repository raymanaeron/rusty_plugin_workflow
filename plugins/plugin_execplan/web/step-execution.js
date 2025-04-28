let ws;

// Initialize WebSocket connection
function initializeWebSocket() {
    ws = new WebSocket('ws://localhost:8081/ws');
    
    ws.onopen = () => {
        ws.send('register-name:execution_ui');
        console.log('[execution] Connected to WebSocket server');
    };

    ws.onerror = (error) => {
        console.error('[execution] WebSocket error:', error);
    };

    ws.onclose = () => {
        console.log('[execution] Disconnected from WebSocket server');
        setTimeout(initializeWebSocket, 2000);
    };
}

// Get form data
function getFormData() {
    // Implement form data collection based on your UI
    return {
        field1: document.getElementById('field1').value,
        field2: document.getElementById('field2').checked,
    };
}

// Set form data
function setFormData(data) {
    // Implement form data setting based on your UI
    document.getElementById('field1').value = data.field1;
    document.getElementById('field2').checked = data.field2;
}

// Show status message
function showStatus(message, isError = false) {
    const statusEl = document.getElementById('statusMessage');
    statusEl.textContent = message;
    statusEl.className = `alert mt-3 ${isError ? 'alert-danger' : 'alert-success'}`;
    statusEl.style.display = 'block';
    setTimeout(() => statusEl.style.display = 'none', 3000);
}

// Load current data
async function loadData() {
    try {
        const response = await fetch('/api/execution/blueprint');
        if (response.ok) {
            const data = await response.json();
            setFormData(data);
        } else {
            showStatus('Failed to load data', true);
        }
    } catch (error) {
        console.error('Error loading data:', error);
        showStatus('Error loading data', true);
    }
}

export async function activate(container) {
    // Initialize WebSocket
    initializeWebSocket();

    // Load current data
    await loadData();

    // Handle form submission
    const form = container.querySelector('#dataForm');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const data = getFormData();

        try {
            const response = await fetch('/api/execution/blueprint', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(data)
            });

            if (response.ok) {
                showStatus('Data saved successfully');

                // Publish update event
                if (ws && ws.readyState === WebSocket.OPEN) {
                    const message = {
                        publisher_name: "execution_ui",
                        topic: "blueprintUpdated",
                        payload: JSON.stringify(data),
                        timestamp: new Date().toISOString()
                    };
                    ws.send(`publish-json:${JSON.stringify(message)}`);
                }
            } else {
                showStatus('Failed to save data', true);
            }
        } catch (error) {
            console.error('Error saving data:', error);
            showStatus('Error saving data', true);
        }
    });

    // Handle reset button
    const resetBtn = container.querySelector('#resetBtn');
    resetBtn.addEventListener('click', async () => {
        try {
            const response = await fetch('/api/execution/blueprint', {
                method: 'DELETE'
            });

            if (response.ok) {
                await loadData();
                showStatus('Data reset to defaults');
            } else {
                showStatus('Failed to reset data', true);
            }
        } catch (error) {
            console.error('Error resetting data:', error);
            showStatus('Error resetting data', true);
        }
    });
}
