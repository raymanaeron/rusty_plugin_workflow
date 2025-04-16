// webapp/plugin-loader.js

export async function loadPluginView(pluginName, container) {
    const htmlUrl = `/${pluginName}/web/step-${pluginName}.html`;
    const jsUrl = `/${pluginName}/web/step-${pluginName}.js`;
  
    try {
      const html = await fetch(htmlUrl).then(res => {
        if (!res.ok) throw new Error(`Failed to load ${htmlUrl}`);
        return res.text();
      });
  
      container.innerHTML = html;
  
      const module = await import(jsUrl);
      if (typeof module.activate === "function") {
        await module.activate(container);
      } else {
        console.warn(`No activate() function exported by ${jsUrl}`);
      }
    } catch (err) {
      console.error("Plugin load error:", err);
      container.innerHTML = `
        <div class="alert alert-danger">
          Failed to load plugin view.<br>
          ${err.message}
        </div>`;
    }
  }
  