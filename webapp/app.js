// webapp/app.js
import { routeTo } from './router.js';

document.addEventListener("DOMContentLoaded", () => {
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
