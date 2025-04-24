let ws; // Declare WebSocket variable in scope
let statusContent;

// Initialize WebSocket connection
function initializeWebSocket() {
    ws = new WebSocket('ws://localhost:8081/ws');
    
    ws.onopen = () => {
        ws.send('register-name:plugin_status');
        console.log('[plugin_status] Connected to WebSocket server');

        // Subscribe to StatusMessageChanged topic (not StatusMessageReceived)
        ws.send('subscribe:StatusMessageChanged');
        console.log('[plugin_status] Subscribed to StatusMessageChanged topic');
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            if (data.topic === 'StatusMessageChanged') {
                console.log('[plugin_status] Received status update:', data.payload);
                statusContent.textContent = data.payload || 'Unknown status';
            }
        } catch (err) {
            console.error('[plugin_status] Error processing message:', err);
            statusContent.textContent = 'Error processing status update';
        }
    };

    ws.onerror = (error) => {
        console.error('[plugin_status] WebSocket error:', error);
        statusContent.textContent = 'Connection error';
        
        // Try to reconnect after error
        setTimeout(initializeWebSocket, 2000);
    };

    ws.onclose = () => {
        console.log('[plugin_status] Disconnected from WebSocket server');
        statusContent.textContent = 'Connection closed';
        
        // Try to reconnect after close
        setTimeout(initializeWebSocket, 2000);
    };
}

export function activate(container) {
    statusContent = container.querySelector("#statusContent");
    if (!statusContent) {
        console.error('[plugin_status] Status content element not found');
        return;
    }

    // Initialize WebSocket when the module activates
    initializeWebSocket();
}

