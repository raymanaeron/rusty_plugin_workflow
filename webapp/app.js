// webapp/app.js
import { routeTo } from './router.js';
import { wsManager } from './websocket_connection_manager.js';

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
    // Register app with ws manager
    wsManager.registerPlugin('app');
    
    // Subscribe to route changes
    wsManager.subscribe('app', 'SwitchRoute', (data) => {
        handleRouting(data.payload);
    });

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
