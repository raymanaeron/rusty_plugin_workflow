export async function activate(container, appManager, jwtManager) {
    appManager.registerPlugin('plugin_welcome');
    console.log('Plugin activated: plugin_welcome');
    const welcomeContent = container.querySelector('#welcomeContent');

    // Initialize WebSocket
    // appManager.initializeWebSocket();

    getWelcomeMessage()

    const getStartedBtn = container.querySelector('#getStartedBtn');
    console.log("getStartedBtn:", getStartedBtn); 
    if (getStartedBtn) {
        getStartedBtn.addEventListener('click', () => {
            // Publish via connection manager
            appManager.publish('plugin_welcome', 'WelcomeCompleted', '{"status": "completed"}');

            // Temporary redirect to /wifi/web
            // appManager.publish('plugin_welcome', 'SwitchRoute', '/wifi/web');
        });
    }

    async function getWelcomeMessage() {
        const res = await jwtManager.secure_request("/api/welcome/welcomemessage");
        if (!res.ok) throw new Error(`Fetch welcome message failed (${res.status})`);
        const welcomeMsg = await res.json();

        // Update only the #welcomeText div, not the whole #welcomeContent
        const welcomeText = welcomeContent.querySelector('#welcomeText');
        if (welcomeText) {
            welcomeText.textContent = welcomeMsg.message;
        }
        console.log(welcomeMsg.message);
    }

    // Remove this line
    // appManager.publish('plugin_welcome', 'SwitchRoute', '/wifi/web');

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_welcome');
    };
}