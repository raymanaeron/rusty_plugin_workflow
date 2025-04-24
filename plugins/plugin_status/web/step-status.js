const statusContent = document.getElementById('statusContent');

// Create WebSocket connection
const ws = new WebSocket('ws://localhost:8081/ws');

// Initialize WebSocket client
ws.onopen = () => {
    // Register client name
    ws.send('register-name:plugin_status');
    console.log('Connected to WebSocket server');

    // Subscribe to StatusMessageReceived topic
    ws.send('subscribe:StatusMessageReceived');
    console.log('Subscribed to StatusMessageReceived topic');
};

// Handle incoming messages
ws.onmessage = (event) => {
    try {
        const data = JSON.parse(event.data);
        if (data.topic === 'StatusMessageReceived') {
            // Parse the payload which should contain the status
            const status = JSON.parse(data.payload);
            statusContent.textContent = status.status || 'Unknown status';
        }
    } catch (err) {
        console.error('Error processing message:', err);
        statusContent.textContent = 'Error processing status update';
    }
};

// Handle WebSocket errors
ws.onerror = (error) => {
    console.error('WebSocket error:', error);
    statusContent.textContent = 'Connection error';
};

// Handle WebSocket connection close
ws.onclose = () => {
    console.log('Disconnected from WebSocket server');
    statusContent.textContent = 'Connection closed';
};

