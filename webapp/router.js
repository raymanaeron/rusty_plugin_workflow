// webapp/router.js
export async function routeTo(path) {
    const container = document.getElementById("content");
    container.innerHTML = `<div class="text-muted">Loading...</div>`;
  
    try {
      // Example: /wifi/web -> wifi
      const parts = path.split("/").filter(Boolean);
      const pluginName = parts[0];
  
      if (!pluginName) {
        container.innerHTML = `<div class="alert alert-info">Welcome! Please select a plugin.</div>`;
        return;
      }
  
      const modulePath = `/${pluginName}/web/step-${pluginName}.js`;
  
      const module = await import(modulePath);
      if (typeof module.render === "function") {
        await module.render(container);
      } else {
        container.innerHTML = `<div class="alert alert-danger">Plugin ${pluginName} does not export a render() function.</div>`;
      }
    } catch (err) {
      console.error("Plugin load error:", err);
      container.innerHTML = `<div class="alert alert-danger">Failed to load plugin view.</div>`;
    }
  }
  