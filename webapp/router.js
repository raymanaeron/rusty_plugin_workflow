import { wsManager } from './websocket_connection_manager.js';

export async function routeTo(path) {
  const container = document.getElementById("content");
  container.innerHTML = `<div class="text-muted">Loading...</div>`;

  const parts = path.split("/").filter(Boolean);
  const pluginName = parts[0];  // Will be "terms" from "/terms/web"

  console.log('[Router] Parts:', parts);
  console.log('[Router] Routing to path:', path);
  console.log('[Router] Plugin name:', pluginName);
  console.log('[Router] Current location:', window.location.href);

  // Ignore root or accidental /index.html navigation
  if (!pluginName || pluginName === "index.html") {
    container.innerHTML = `<div class="alert alert-info">Welcome! Please select a plugin.</div>`;
    return;
  }

  try {
    // Keep /web in the URLs to match server paths
    const basePath = `/${pluginName}/web`;
    const htmlUrl = `${basePath}/step-${pluginName}.html`;
    const jsUrl = `${basePath}/step-${pluginName}.js`;

    console.log('[Router] Base path:', basePath);
    console.log('[Router] HTML URL:', htmlUrl);
    console.log('[Router] JS URL:', jsUrl);
    console.log('[Router] Document base URI:', document.baseURI);

    const response = await fetch(htmlUrl, {
      headers: {
        'Accept': 'text/html',
        'Cache-Control': 'no-cache'
      },
      // Add cache busting query param
      cache: 'no-store'
    });
    
    console.log('[Router] HTML fetch response:', response.status, response.statusText);
    
    if (!response.ok) {
      throw new Error(`Failed to load ${htmlUrl} (${response.status} ${response.statusText})`);
    }

    const html = await response.text();
    container.innerHTML = html;

    console.log('[Router] Attempting to load JS module from:', jsUrl);
    
    const module = await import(jsUrl);
    if (module.activate) {
      await module.activate(container, wsManager);  // Inject wsManager
    } else {
      console.warn(`Plugin ${pluginName} has no activate() function`);
    }
  } catch (err) {
    console.error('[Router] Error details:', err);
    container.innerHTML = `<div class="alert alert-danger">
      Failed to load plugin view.<br>
      ${err.message}<br>
      <small class="text-muted">Check browser console for details.</small>
    </div>`;
  }
}
