export async function activate(container, appManager) {
    // Register with app manager
    appManager.registerPlugin('plugin_login');
    console.log('Plugin activated: plugin_login');
    
    // Get UI elements
    const statusContent = container.querySelector('#statusContent');
    const actionBtn = container.querySelector('#actionBtn');
    const skipBtn = container.querySelector('#skipBtn');
    const continueBtn = container.querySelector('#continueBtn');
    const resultBox = container.querySelector('#resultBox');
    
    // Example: GET data from API
    async function getData() {
        try {
            const response = await fetch('/api/login/userprofile');
            if (response.ok) {
                const data = await response.json();
                console.log('Data loaded:', data);
                return data;
            } else {
                console.error('Failed to load data:', response.statusText);
                throw new Error(`Failed to load data: ${response.statusText}`);
            }
        } catch (error) {
            console.error('Error loading data:', error);
            throw error;
        }
    }
    
    // Example: POST data to API
    async function postData(payload) {
        try {
            const response = await fetch('/api/login/userprofile', {
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
    
    // Example: PUT data to API (update)
    async function putData(id, payload) {
        try {
            const response = await fetch(`/api/login/userprofile/${id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            
            const data = await response.json();
            if (response.ok) {
                console.log('Data updated successfully:', data);
                return data;
            } else {
                console.error('Failed to update data:', data);
                throw new Error(data.message || 'Failed to update data');
            }
        } catch (error) {
            console.error('Error updating data:', error);
            throw error;
        }
    }
    
    // Example: PATCH data to API (partial update)
    async function patchData(id, partialPayload) {
        try {
            const response = await fetch(`/api/login/userprofile/${id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(partialPayload)
            });
            
            const data = await response.json();
            if (response.ok) {
                console.log('Data patched successfully:', data);
                return data;
            } else {
                console.error('Failed to patch data:', data);
                throw new Error(data.message || 'Failed to patch data');
            }
        } catch (error) {
            console.error('Error patching data:', error);
            throw error;
        }
    }
    
    // Example: Action button handler
    if (actionBtn) {
        actionBtn.addEventListener('click', async () => {
            try {
                resultBox.innerHTML = '<div class="alert alert-info">Processing...</div>';
                const result = await getData();
                resultBox.innerHTML = '<div class="alert alert-success">Action completed!</div>';
            } catch (error) {
                resultBox.innerHTML = `<div class="alert alert-danger">${error.message}</div>`;
            }
        });
    }
    
    // Example: Skip button handler
    if (skipBtn) {
        skipBtn.addEventListener('click', async () => {
            // Publish via connection manager using CamelCased event topic
            const published = appManager.publish('plugin_login', 'UserprofileCompleted', 
                { status: 'skipped' }
            );
            
            if (published) {
                console.log("[plugin_login] Skip status published");
                resultBox.innerHTML = '<div class="alert alert-info">Setup skipped. Redirecting...</div>';
            } else {
                console.warn("[plugin_login] Skip publish failed");
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to publish skip status</div>';
            }
        });
    }
    
    // Example: Continue button handler
    if (continueBtn) {
        continueBtn.addEventListener('click', async () => {
            // Publish completion event using CamelCased event topic
            const published = appManager.publish('plugin_login', 'UserprofileCompleted', 
                { status: 'completed' }
            );
            
            if (published) {
                console.log("[plugin_login] Completion status published");
            } else {
                console.warn("[plugin_login] Completion publish failed");
            }
        });
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_login');
    };
}
