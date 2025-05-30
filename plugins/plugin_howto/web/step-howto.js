export async function activate(container, appManager, jwtManager) {
    // Register this plugin with the application manager for lifecycle management
    appManager.registerPlugin('plugin_howto');
    console.log('Plugin activated: plugin_howto');
    
    // Find and cache DOM elements for UI interaction
    const statusContent = container.querySelector('#statusContent');
    const actionBtn = container.querySelector('#actionBtn');
    const skipBtn = container.querySelector('#skipBtn');
    const continueBtn = container.querySelector('#continueBtn');
    const resultBox = container.querySelector('#resultBox');
    
    // Fetch all resources from the plugin's REST API endpoint
    // Returns a promise with the retrieved data or throws an error
    async function getData() {
        try {
            const response = await fetch('/api/howto/todoitems');
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
    
    // Create a new resource by sending data to the plugin's REST API
    // payload: Object to be JSON-serialized and sent to the server
    // Returns the server's response or throws an error
    async function postData(payload) {
        try {
            const response = await fetch('/api/howto/todoitems', {
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
    
    // Update an existing resource by sending data to the plugin's REST API
    // id: Identifier of the resource to update
    // payload: Object to be JSON-serialized and sent to the server
    // Returns the server's response or throws an error
    async function putData(id, payload) {
        try {
            const response = await fetch(`/api/howto/todoitems/${id}`, {
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
    
    // Partially update an existing resource by sending data to the plugin's REST API
    // id: Identifier of the resource to update
    // partialPayload: Object to be JSON-serialized and sent to the server
    // Returns the server's response or throws an error
    async function patchData(id, partialPayload) {
        try {
            const response = await fetch(`/api/howto/todoitems/${id}`, {
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
    
    // Handle the action button click event
    // Fetches data from the server and updates the UI accordingly
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
    
    // Handle the skip button click event
    // Publishes a skip status event and updates the UI accordingly
    if (skipBtn) {
        skipBtn.addEventListener('click', async () => {
            // Publish via connection manager 
            const published = appManager.publish('plugin_howto', 'HowtoCompleted', 
                { status: 'skipped' }
            );
            
            if (published) {
                console.log("[plugin_howto] Skip status published");
                resultBox.innerHTML = '<div class="alert alert-info">Setup skipped. Redirecting...</div>';
            } else {
                console.warn("[plugin_howto] Skip publish failed");
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to publish skip status</div>';
            }
        });
    }
    
    // Handle the continue button click event
    // Publishes a completion status event
    if (continueBtn) {
        continueBtn.addEventListener('click', async () => {
            // Publish completion event 
            const published = appManager.publish('plugin_howto', 'HowtoCompleted', 
                { status: 'completed' }
            );
            
            if (published) {
                console.log("[plugin_howto] Completion status published");
            } else {
                console.warn("[plugin_howto] Completion publish failed");
            }
        });
    }

    // Return cleanup function at module level
    // Unregisters the plugin from the application manager
    return () => {
        appManager.unregisterPlugin('plugin_howto');
    };
}