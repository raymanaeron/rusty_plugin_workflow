import { routeTo } from './router.js';
import { appManager } from './app_manager.js';
import * as jwtManager from './jwt_manager.js';

function handleRouting(forcePath = null) {
  // Handle root redirects
  if (location.pathname === "/" || location.pathname === "/index.html") {
    history.replaceState({}, "", "/welcome/web");
  }

  // Route to either forced path or current location
  const targetPath = forcePath || location.pathname;
  routeTo(targetPath, appManager, jwtManager);
}

document.addEventListener("DOMContentLoaded", async () => {
    // Initialize WebSocket
    // appManager.initializeWebSocket(); // Remove this line

    // Register app with ws manager
    appManager.registerPlugin('app');

    // Subscribe to route changes - this is correct as SWITCH_ROUTE is "SwitchRoute" in the engine
    appManager.subscribe('app', 'SwitchRoute', (data) => {
        console.log("App -->> SwitchRoute event received:", data);
        // Add more debugging to verify payload structure
        console.log("Payload type:", typeof data.payload, "Value:", data.payload);

        if (data && data.payload) {
            handleRouting(data.payload);
        } else {
            console.error("Received SwitchRoute event with invalid payload structure:", data);
        }
    });
    console.log("App: Subscribed to SwitchRoute.  Waiting for messages...");

    // Add debug logging for all WebSocket messages
    console.log("WebSocket connection setup complete. Waiting for messages...");

    // Setup route handling
    try {
        // Initialize JWT authentication first
        await jwtManager.initialize_with_jwt();
        console.log('JWT authentication initialized successfully');
        
        // Then proceed with initial routing after JWT is ready
        handleRouting();
    } catch (error) {
        console.error('Failed to initialize JWT authentication:', error);
        // Still attempt routing even if JWT initialization fails
        handleRouting();
    }

    // Signal that app is ready
    // appManager.setReady(); // Remove this line

    // Handle forward/back nav
});