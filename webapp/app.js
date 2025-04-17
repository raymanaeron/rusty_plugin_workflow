// webapp/app.js
import { routeTo } from './router.js';

document.addEventListener("DOMContentLoaded", () => {
  // redirect block before initial route load
  if (location.pathname === "/" || location.pathname === "/index.html") {
    history.replaceState({}, "", "/terms/web");
  }

  // Handle forward/back nav
  window.addEventListener("popstate", () => {
    routeTo(location.pathname);
  });

  // Initial route load
  routeTo(location.pathname);

  // Exit button behavior
  const exitBtn = document.getElementById("exitBtn");
  if (exitBtn) {
    exitBtn.addEventListener("click", () => {
      // Simulate graceful exit — replace with real behavior as needed
      alert("Exiting Device Setup");
    });
  }
});
