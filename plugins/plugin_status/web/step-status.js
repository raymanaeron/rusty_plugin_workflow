export async function activate(container, wsManager) {
    wsManager.registerPlugin('plugin_status');
    const statusContent = container.querySelector('#statusContent');

    // Subscribe to status updates
    wsManager.subscribe('plugin_status', 'StatusMessageChanged', (data) => {
        statusContent.textContent = data.payload || 'Unknown status';
    });

    // Cleanup on deactivate
    return () => {
        wsManager.unregisterPlugin('plugin_status');
        wsManager.unsubscribe('plugin_status', 'StatusMessageChanged');
    };
}

