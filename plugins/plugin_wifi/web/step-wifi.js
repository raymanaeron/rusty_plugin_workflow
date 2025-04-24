const next_route = "/status/web";
let ws; // Declare WebSocket variable in scope

// Initialize WebSocket connection
function initializeWebSocket() {
    ws = new WebSocket('ws://localhost:8081/ws');
    
    ws.onopen = () => {
        ws.send('register-name:plugin_wifi');
        console.log('WiFi plugin connected to WebSocket server');
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
        console.log('Disconnected from WebSocket server');
    };
}

// Returns the current timestamp in ISO format
function getTimestamp() {
    return new Date().toISOString();
}

export async function activate(container) {
    // Initialize WebSocket when the module activates
    initializeWebSocket();

    const scanBtn = container.querySelector("#scanBtn");
    const connectBtn = container.querySelector("#connectBtn");
    const resultBox = container.querySelector("#resultBox");
    const networkList = container.querySelector("#networkList");
    const passwordInput = container.querySelector("#password");
    const scanStatus = container.querySelector("#scanStatus");
  
    scanBtn.addEventListener("click", async () => {
        resultBox.innerHTML = "";
        scanStatus.textContent = "Scanning...";
        scanBtn.disabled = true;
    
        try {
            const res = await fetch("/api/wifi/network");
            if (!res.ok) throw new Error(`Scan failed (${res.status})`);
            const networks = await res.json();
    
            networkList.innerHTML = `<option value="" disabled selected>-- Choose a network --</option>`;
            networks.forEach(n => {
                const opt = document.createElement("option");
                opt.value = n.ssid;
                opt.textContent = `${n.ssid} (${n.signal} dBm)`;
                networkList.appendChild(opt);
            });
    
            scanStatus.textContent = `Found ${networks.length} network(s)`;
        } catch (err) {
            scanStatus.textContent = "Scan failed";
            resultBox.innerHTML = `<div class="alert alert-danger">${err.message}</div>`;
        } finally {
            scanBtn.disabled = false;
        }
    });
  
    connectBtn.addEventListener("click", async () => {
        const ssid = networkList.value;
        const password = passwordInput.value;
        resultBox.innerHTML = "";
    
        if (!ssid) {
            resultBox.innerHTML = `<div class="alert alert-warning">Please select a network.</div>`;
            return;
        }
    
        if (!password) {
            resultBox.innerHTML = `<div class="alert alert-warning">Please enter a password.</div>`;
            return;
        }
    
        connectBtn.disabled = true;
    
        try {
            const res = await fetch("/api/wifi/network", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ ssid, password }),
            });
    
            let json;

            try {
                json = await res.json();
            } catch (err) {
                const fallback = await res.text();
                throw new Error(fallback); // Will show "Resource not found"
            }

            if (res.ok) {
                resultBox.innerHTML = `<div class="alert alert-success">${json.message || "Connected successfully"}</div>`;

                // Publish network connected message with retries
                if (ws && ws.readyState === WebSocket.OPEN) {
                    const message = {
                        publisher_name: "plugin_wifi",
                        topic: "NetworkConnected",
                        payload: JSON.stringify({ status: 'connected', ssid: ssid }),
                        timestamp: getTimestamp()
                    };

                    let retries = 3;
                    const publishWithRetry = () => {
                        try {
                            console.log(`[publish] plugin_wifi sending to ${message.topic} with payload=${message.payload}`);
                            ws.send(`publish-json:${JSON.stringify(message)}`);
                            
                            // Navigate after successful publish
                            console.log("[publish] Message sent, navigating to status page...");
                            setTimeout(() => {
                                history.pushState({}, "", next_route);
                                window.dispatchEvent(new PopStateEvent("popstate"));
                            }, 2000); // Increased timeout for better reliability
                        } catch (error) {
                            console.error("[publish] Error sending message:", error);
                            if (retries > 0) {
                                retries--;
                                setTimeout(publishWithRetry, 500);
                            }
                        }
                    };

                    publishWithRetry();
                } else {
                    console.warn("[publish] WebSocket not ready, navigating without status update");
                    setTimeout(() => {
                        history.pushState({}, "", next_route);
                        window.dispatchEvent(new PopStateEvent("popstate"));
                    }, 1000);
                }
            } else {
                resultBox.innerHTML = `<div class="alert alert-danger">${json.message || "Connection failed"}</div>`;
            }
        } catch (err) {
            resultBox.innerHTML = `<div class="alert alert-danger">${err.message}</div>`;
        } finally {
            connectBtn.disabled = false;
        }
    });
}
