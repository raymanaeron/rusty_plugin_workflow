export async function routeTo(path) {
  const container = document.getElementById("content");
  container.innerHTML = `<div class="text-muted">Loading...</div>`;

  const parts = path.split("/").filter(Boolean);
  const pluginName = parts[0];

  // Ignore root or accidental /index.html navigation
  if (!pluginName || pluginName === "index.html") {
    container.innerHTML = `<div class="alert alert-info">Welcome! Please select a plugin.</div>`;
    return;
  }

  try {
    const htmlUrl = `/${pluginName}/web/step-${pluginName}.html`;
    const jsUrl = `/${pluginName}/web/step-${pluginName}.js`;

    const html = await fetch(htmlUrl).then(res => {
      if (!res.ok) throw new Error(`Failed to load ${htmlUrl}`);
      return res.text();
    });

    container.innerHTML = html;

    const module = await import(jsUrl);
    if (typeof module.activate === "function") {
      await module.activate(container);
    } else {
      console.warn(`Plugin ${pluginName} has no activate() function`);
    }
  } catch (err) {
    console.error("Plugin load error:", err);
    container.innerHTML = `<div class="alert alert-danger">Failed to load plugin view.<br>${err.message}</div>`;
  }
}
