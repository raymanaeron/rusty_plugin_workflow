// webapp/app.js
import { routeTo } from './router.js';

let ws;

// Initialize WebSocket connection
function initializeWebSocket() {
  ws = new WebSocket('ws://localhost:8081/ws');

  ws.onopen = () => {
    ws.send('register-name:app');
    console.log('[webapp] Connected to WebSocket server');

    // Subscribe to SwitchRoute topic 
    ws.send('subscribe:SwitchRoute');
    console.log('[webapp] Subscribed to SwitchRoute topic');
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.topic === 'SwitchRoute') {
        console.log('[webapp] Received SwitchRoute update:', data.payload);
        handleRouting(data.payload);  // Call with forced path from WebSocket
      }
    } catch (err) {
      console.error('[webapp] Error processing message:', err);
    }
  };

  ws.onerror = (error) => {
    console.error('[webapp] WebSocket error:', error);

    // Try to reconnect after error
    setTimeout(initializeWebSocket, 2000);
  };

  ws.onclose = () => {
    console.log('[webapp] Disconnected from WebSocket server');

    // Try to reconnect after close
    setTimeout(initializeWebSocket, 2000);
  };
}

function handleRouting(forcePath = null) {
  // Handle root redirects
  if (location.pathname === "/" || location.pathname === "/index.html") {
    history.replaceState({}, "", "/terms/web");
  }

  // Route to either forced path or current location
  const targetPath = forcePath || location.pathname;
  routeTo(targetPath);
}

document.addEventListener("DOMContentLoaded", () => {
  // Initialize WebSocket
  initializeWebSocket();

  // Setup route handling
  handleRouting();  // Initial route handling

  // Handle forward/back nav
  window.addEventListener("popstate", () => {
    handleRouting();
  });

  // Exit button behavior
  const exitBtn = document.getElementById("exitBtn");
  if (exitBtn) {
    exitBtn.addEventListener("click", () => {
      // Simulate graceful exit — replace with real behavior as needed
      alert("Exiting Device Setup");
    });
  }
});
