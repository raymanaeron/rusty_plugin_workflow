import { network_connected, goToNextStep } from './app.js';

export function renderWiFiPage(container) {
  const section = document.createElement('section');
  section.innerHTML = `
    <h1>WiFi Setup</h1>
    <button id="scan-btn">Scan WiFi</button>
    <select id="wifi-list" class="listbox"></select>
    <input id="wifi-password" type="password" placeholder="Enter WiFi password" />
    <button id="connect-btn">Connect</button>
    <button id="next-btn" disabled>Next</button>
  `;

  container.appendChild(section);

  const scanBtn = document.getElementById('scan-btn');
  const wifiList = document.getElementById('wifi-list');
  const connectBtn = document.getElementById('connect-btn');
  const nextBtn = document.getElementById('next-btn');

  scanBtn.addEventListener('click', async () => {
    wifiList.innerHTML = '';
    try {
      const res = await fetch('/api/wifi/scan');
      const data = await res.json();
      data.forEach((net, i) => {
        const opt = document.createElement('option');
        opt.value = net.ssid;
        opt.text = `${net.ssid} (${net.signal} dBm)`;
        wifiList.appendChild(opt);
      });
    } catch (err) {
      alert('Failed to scan WiFi');
    }
  });

  connectBtn.addEventListener('click', () => {
    const ssid = wifiList.value;
    const password = document.getElementById('wifi-password').value;
    if (!ssid || !password) {
      alert('Please select a network and enter a password.');
      return;
    }

    // For now, assume connection succeeds
    console.log(`Pretend connecting to ${ssid} with ${password}`);
    window.network_connected = true;
    nextBtn.disabled = false;
  });

  nextBtn.addEventListener('click', () => {
    goToNextStep();
  });
}
