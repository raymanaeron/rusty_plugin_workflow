// plugins/plugin_wifi/web/step-wifi.js

export async function activate(container) {
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
        const res = await fetch("/wifi/api/network");
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
        const res = await fetch("/wifi/api/network", {
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
  