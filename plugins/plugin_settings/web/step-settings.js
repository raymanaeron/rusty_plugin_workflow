export async function activate(container, appManager) {
    // Register with app manager
    appManager.registerPlugin('plugin_settings');
    console.log('Plugin activated: plugin_settings');

    // Get UI elements
    const submitBtn = container.querySelector('#submitBtn');
    const clearBtn = container.querySelector('#clearBtn');

    async function postData(payload) {
        try {
            const response = await fetch('/api/settings/devicesettings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            const data = await response.json();
            if (response.ok) {
                console.log('Data posted successfully:', data);
                return data;
            } else {
                console.error('Failed to post data:', data);
                throw new Error(data.message || 'Failed to post data');
            }
        } catch (error) {
            console.error('Error posting data:', error);
            throw error;
        }
    }

    if (clearBtn) {
        clearBtn.addEventListener('click', () => {

        });
    }

    if (submitBtn) {
        submitBtn.addEventListener('click', async () => {


            // Example: POST data to API
            try {

            } catch (error) {
                statusContent.innerHTML = `Save settings failed: ${error.message}`;
            }

            const published = appManager.publish('plugin_settings', 'SettingsCompleted',
                { status: 'completed' }
            );
        });
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_settings');
    }
}
