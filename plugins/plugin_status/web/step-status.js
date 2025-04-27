export async function activate(container, appManager) {
    appManager.registerPlugin('plugin_status');
    const statusContent = container.querySelector('#statusContent');

    // Subscribe to status updates
    appManager.subscribe('plugin_status', 'StatusMessageChanged', (data) => {
        statusContent.textContent = data.payload || 'Unknown status';
    });

    // Cleanup on deactivate
    return () => {
        appManager.unregisterPlugin('plugin_status');
        appManager.unsubscribe('plugin_status', 'StatusMessageChanged');
    };
}

