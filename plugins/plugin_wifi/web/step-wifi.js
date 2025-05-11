const next_route = "/status/web";

export async function activate(container, appManager, jwtManager) {
    // Register with connection manager
    appManager.registerPlugin('plugin_wifi');

    const scanBtn = container.querySelector("#scanBtn");
    const connectBtn = container.querySelector("#connectBtn");
    const skipBtn = container.querySelector("#skipBtn");
    const resultBox = container.querySelector("#resultBox");
    const networkList = container.querySelector("#networkList");
    const passwordInput = container.querySelector("#password");
    const scanStatus = container.querySelector("#scanStatus");
    const networkListBox = container.querySelector("#networkListBox");

    // Helper to map signal strength (typically negative dBm values) to icon filename
    function getSignalIconName(signal) {
        // Signal strength is typically in dBm (-30 to -90)
        // Higher (less negative) values mean stronger signal
        // Strong: -30 to -50 dBm
        // Good: -50 to -60 dBm
        // Fair: -60 to -70 dBm
        // Weak: less than -70 dBm
        if (signal >= -50) return "/wifi/web/icons/wifi-strong.svg";
        if (signal >= -60) return "/wifi/web/icons/wifi-good.svg";
        if (signal >= -70) return "/wifi/web/icons/wifi-fair.svg";
        return "/wifi/web/icons/wifi-weak.svg";
    }

    async function getNetworkList() {
        try {
            scanBtn.disabled = true;
            scanBtn.classList.add("loading");
            scanStatus.innerHTML = "Scanning networks...";
            networkList.innerHTML = "";
            
            // Use secure_request from jwtManager instead of fetch
            const response = await jwtManager.secure_request('/api/wifi/network');
            if (!response.ok) throw new Error(`Network scan failed (${response.status})`);
            
            const networks = await response.json();
            
            if (!Array.isArray(networks) || networks.length === 0) {
                scanStatus.innerHTML = "No networks found";
                scanBtn.disabled = false;
                scanBtn.classList.remove("loading");
                return;
            }
            
            scanStatus.innerHTML = `${networks.length} network(s) found`;
            networkList.innerHTML = "";
            
            let selectedNetwork = null;

            networks.forEach(n => {
                const li = document.createElement("li");
                
                // Use the existing getSignalIconName function to get the appropriate icon
                const iconPath = getSignalIconName(n.signal);
                
                // Use daisyUI menu item style without circle background
                li.innerHTML = `
                    <button class="network-item w-full text-left flex items-center gap-4 p-3 rounded-lg" data-ssid="${n.ssid}">
                      <img src="${iconPath}" alt="Signal strength" class="h-6 w-6">
                      <div class="flex-1">
                        <div class="font-bold">${n.ssid}</div>
                        <div class="text-sm opacity-70">${n.security || "Open"}</div>
                      </div>
                      <div class="text-sm opacity-70">${n.signal} dBm</div>
                    </button>
                `;
                
                // Add click handler directly to the button
                const networkItem = li.querySelector('.network-item');
                networkItem.addEventListener('click', () => {
                    // Store selected network SSID
                    selectedNetwork = n.ssid;
                    
                    // Remove active class from all items
                    document.querySelectorAll('.network-item').forEach(item => {
                        item.classList.remove('bg-primary', 'text-primary-content');
                    });
                    
                    // Add active class to selected item and keep it highlighted
                    networkItem.classList.add('bg-primary', 'text-primary-content');
                    
                    // Enable connect button
                    connectBtn.disabled = false;
                });
                
                // Append to list
                networkList.appendChild(li);
            });
            
            scanBtn.disabled = false;
            scanBtn.classList.remove("loading");
            connectBtn.disabled = true; // Initially disable connect button until network is selected
            
        } catch (err) {
            scanStatus.innerHTML = `Error: ${err.message}`;
            scanBtn.disabled = false;
            scanBtn.classList.remove("loading");
            console.error(err);
        }
    }

    scanBtn.addEventListener("click", async () => {
        await getNetworkList();
    });
    skipBtn.addEventListener("click", async () => {
        resultBox.innerHTML = "";
        skipBtn.disabled = true;
        // Tell the engine that we are done now
        const published = appManager.publish('plugin_wifi', 'WifiCompleted', 
            { status: 'skipped' }
        );
        if (published) {
            resultBox.innerHTML = `<div class="alert alert-info">WiFi setup skipped. Redirecting...</div>`;
            console.log("[plugin_wifi] Network skip published, navigating...");
        } else {
            resultBox.innerHTML = `<div class="alert alert-warning">Skip failed to publish. Redirecting anyway...</div>`;
            console.warn("[plugin_wifi] Skip publish failed");
        }
        // Navigate to next route
        setTimeout(() => {
            history.pushState({}, "", next_route);
            window.dispatchEvent(new PopStateEvent("popstate"));
        }, 2000);
    });     history.pushState({}, "", next_route);
            window.dispatchEvent(new PopStateEvent("popstate"));
    connectBtn.addEventListener("click", async () => {
        const selectedNetworkItem = document.querySelector('.network-item.bg-primary');
        if (!selectedNetworkItem) {
            resultBox.innerHTML = `<div class="alert alert-warning">Please select a network first</div>`;
            return;
        }
        
        const ssid = selectedNetworkItem.getAttribute('data-ssid');
        const password = passwordInput.value;
        resultBox.innerHTML = "";
    
        if (!ssid) {
            resultBox.innerHTML = `<div class="alert alert-warning">Please select a network.</div>`;
            return;
        }
    
        // Password validation check removed - allow empty passwords

        connectBtn.disabled = true;
        resultBox.innerHTML = `<div class="alert alert-info">Connecting to ${ssid}...</div>`;
        try {
            // Use secure_request from jwtManager instead of fetch
            const res = await jwtManager.secure_request("/api/wifi/network", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ ssid, password }),
            });

            let json;
            try {
                json = await res.json();
            } catch (err) {
                const fallback = await res.text();
                throw new Error(fallback);
            }
            if (res.ok) {
                // Publish via connection manager
                const published = appManager.publish('plugin_wifi', 'WifiCompleted', 
                    { status: 'connected', ssid: ssid }
                );
                if (published) {
                    resultBox.innerHTML = `<div class="alert alert-success">Connected! Redirecting...</div>`;
                    console.log("[plugin_wifi] Network status published, navigating...");
                } else {
                    resultBox.innerHTML = `<div class="alert alert-warning">Connected, but status update failed. Redirecting...</div>`;
                    console.warn("[plugin_wifi] Publish failed, navigating without status update");
                }
                // Single navigation point with longer delay to ensure status is processed
                /*
                setTimeout(() => {
                    history.pushState({}, "", next_route);
                    window.dispatchEvent(new PopStateEvent("popstate"));
                }, 3000);
                */
                window.dispatchEvent(new PopStateEvent("popstate"));
            } else {
                resultBox.innerHTML = `<div class="alert alert-danger">${json.message || "Connection failed"}</div>`;
            }
        } catch (err) {
            resultBox.innerHTML = `<div class="alert alert-danger">${err.message}</div>`;
        } finally {
            connectBtn.disabled = false;
        }
    });

    // On page load, ensure connect button is disabled
    connectBtn.disabled = true;

    // Scan for networks as soon as the controls are ready
    await getNetworkList();

    // Return cleanup function at module level
    // Unregisters the plugin from the application manager
    return () => {
        appManager.unregisterPlugin('plugin_wifi');
    };
}
