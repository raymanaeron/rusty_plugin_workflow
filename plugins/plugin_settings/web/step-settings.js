let ws;

// Initialize WebSocket connection
function initializeWebSocket() {
    ws = new WebSocket('ws://localhost:8081/ws');
    
    ws.onopen = () => {
        ws.send('register-name:plugin_settings_ui');
        console.log('[plugin_settings] Connected to WebSocket server');
    };

    ws.onerror = (error) => {
        console.error('[plugin_settings] WebSocket error:', error);
    };

    ws.onclose = () => {
        console.log('[plugin_settings] Disconnected from WebSocket server');
        setTimeout(initializeWebSocket, 2000);
    };
}

// Get form data as settings object
function getFormData() {
    return {
        timezone: document.getElementById('timezone').value,
        language: document.getElementById('language').value,
        metrics_enabled: document.getElementById('metricsEnabled').checked,
        copy_settings: document.getElementById('copySettings').checked,
        theme: document.getElementById('theme').value
    };
}

// Set form data from settings object
function setFormData(settings) {
    document.getElementById('timezone').value = settings.timezone;
    document.getElementById('language').value = settings.language;
    document.getElementById('metricsEnabled').checked = settings.metrics_enabled;
    document.getElementById('copySettings').checked = settings.copy_settings;
    document.getElementById('theme').value = settings.theme;
}

// Show status message
function showStatus(message, isError = false) {
    const statusEl = document.getElementById('statusMessage');
    statusEl.textContent = message;
    statusEl.className = `alert mt-3 ${isError ? 'alert-danger' : 'alert-success'}`;
    statusEl.style.display = 'block';
    setTimeout(() => statusEl.style.display = 'none', 3000);
}

// Load current settings
async function loadSettings() {
    try {
        const response = await fetch('/api/settings/devicesettings');
        if (response.ok) {
            const settings = await response.json();
            setFormData(settings);
        } else {
            showStatus('Failed to load settings', true);
        }
    } catch (error) {
        console.error('Error loading settings:', error);
        showStatus('Error loading settings', true);
    }
}

export async function activate(container, wsManager) {
    wsManager.registerPlugin('plugin_settings');
    
    // Initialize WebSocket
    initializeWebSocket();

    // Load current settings
    await loadSettings();

    // Handle form submission
    const form = container.querySelector('#settingsForm');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const settings = getFormData();

        try {
            const response = await fetch('/api/settings/devicesettings', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(settings)
            });

            if (response.ok) {
                showStatus('Settings saved successfully');

                // Publish settings update completed event
                if (ws && ws.readyState === WebSocket.OPEN) {
                    const message = {
                        publisher_name: "plugin_settings_ui",
                        topic: "SettingUpdateCompleted",
                        payload: JSON.stringify(settings),
                        timestamp: new Date().toISOString()
                    };
                    ws.send(`publish-json:${JSON.stringify(message)}`);
                }
            } else {
                showStatus('Failed to save settings', true);
            }
        } catch (error) {
            console.error('Error saving settings:', error);
            showStatus('Error saving settings', true);
        }
    });

    // Handle clear button
    const clearBtn = container.querySelector('#clearBtn');
    clearBtn.addEventListener('click', () => {
        const defaultSettings = {
            timezone: 'UTC',
            language: 'en-US',
            metrics_enabled: false,
            copy_settings: false,
            theme: 'light'
        };
        setFormData(defaultSettings);
    });

    // Publish settings changes if needed
    function publishSettingsUpdate(settings) {
        wsManager.publish('plugin_settings', 'SettingsChanged', settings);
    }

    // Cleanup on deactivate
    return () => {
        wsManager.unregisterPlugin('plugin_settings');
    };
}