const next_route = "/status/web";

export async function activate(container, appManager) {
    // Register with connection manager
    appManager.registerPlugin('plugin_wifi');

    const scanBtn = container.querySelector("#scanBtn");
    const connectBtn = container.querySelector("#connectBtn");
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
        resultBox.innerHTML = "";
        scanStatus.textContent = "Scanning...";
        scanBtn.disabled = true;

        try {
            const res = await fetch("/api/wifi/network");
            if (!res.ok) throw new Error(`Scan failed (${res.status})`);
            let networks = await res.json();

            // Filter out networks with no SSID
            networks = networks.filter(n => n.ssid && n.ssid.trim() !== "");

            // Replace dropdown with custom list
            const listBox = networkListBox;
            listBox.innerHTML = "";
            connectBtn.disabled = true; // Disable connect by default after scan
            if (networks.length === 0) {
                listBox.innerHTML = `<li class="list-group-item text-muted">No networks found</li>`;
            } else {
                /*
                    <span class="text-muted small ms-2">
                        ${typeof n.signal === "number" ? n.signal + " dBm" : ""}
                    </span>
                */
                networks.forEach(n => {
                    console.log(n);
                    const iconPath = getSignalIconName(n.signal);
                    const li = document.createElement("li");
                    li.className = "list-group-item d-flex align-items-center";
                    li.tabIndex = 0;
                    li.setAttribute("role", "option");
                    li.setAttribute("data-ssid", n.ssid);
                    li.innerHTML = `
                        <img src="${iconPath}" alt="signal" style="height:1.5em;width:auto;margin-right:0.75em;flex-shrink:0;">
                        <span class="flex-grow-1">${n.ssid}</span>
                        <span class="text-muted small ms-2">${n.security || ""}</span>
                    `;
                    li.addEventListener("click", () => {
                        // Remove selection from others
                        listBox.querySelectorAll(".active").forEach(el => el.classList.remove("active"));
                        li.classList.add("active");
                        networkList.value = n.ssid;
                        connectBtn.disabled = false; // Enable connect when selected
                    });
                    li.addEventListener("keydown", (e) => {
                        if (e.key === "Enter" || e.key === " ") {
                            li.click();
                        }
                    });
                    listBox.appendChild(li);
                });
            }

            scanStatus.textContent = `Found ${networks.length} network(s)`;
        } catch (err) {
            scanStatus.textContent = "Scan failed";
            resultBox.innerHTML = `<div class="alert alert-danger">${err.message}</div>`;
        } finally {
            scanBtn.disabled = false;
        }
    }

    scanBtn.addEventListener("click", async () => {
        await getNetworkList();
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
        resultBox.innerHTML = `<div class="alert alert-info">Connecting to ${ssid}...</div>`;
    
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
                setTimeout(() => {
                    history.pushState({}, "", next_route);
                    window.dispatchEvent(new PopStateEvent("popstate"));
                }, 3000);
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
}
