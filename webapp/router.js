import { appManager } from './app_manager.js';

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
      await module.activate(container, appManager);  // Inject appManager
    } else {
      console.warn(`Plugin ${pluginName} has no activate() function`);
    }
  } catch (err) {
    console.error('[Router] Error loading plugin:', err);
    
    // Detailed error logging to help diagnose WebView vs Browser differences
    console.error('[Router] Error details:', {
      message: err.message,
      stack: err.stack,
      pluginName,
      documentReady: document.readyState,
      containerExists: !!container,
      isWebView: !!(window.webkit || window.android)
    });
    
    // Use a safer approach to render the error that doesn't rely on innerHTML
    try {
      // Clear container first
      while (container.firstChild) {
        container.removeChild(container.firstChild);
      }
      
      // Create alert div
      const alertDiv = document.createElement('div');
      alertDiv.className = 'alert alert-danger';
      
      // Add error message text
      const errorText = document.createTextNode('Failed to load plugin view.');
      alertDiv.appendChild(errorText);
      
      // Add line break
      alertDiv.appendChild(document.createElement('br'));
      
      // Add error message
      const messageText = document.createTextNode(err.message || 'Unknown error');
      alertDiv.appendChild(messageText);
      
      // Add line break
      alertDiv.appendChild(document.createElement('br'));
      
      // Add small text for console info
      const small = document.createElement('small');
      small.className = 'text-muted';
      small.textContent = 'Check browser console for details.';
      alertDiv.appendChild(small);
      
      // Append to container
      container.appendChild(alertDiv);
    } catch (renderErr) {
      // Absolute fallback if DOM manipulation fails
      console.error('[Router] Failed to render error message:', renderErr);
      
      // Try the simplest possible approach
      try {
        container.textContent = `Error: ${err.message || 'Unknown error'}`;
      } catch (finalErr) {
        console.error('[Router] Critical error in error handling:', finalErr);
      }
    }
  }
}
