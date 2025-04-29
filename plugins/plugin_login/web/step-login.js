export async function activate(container, appManager) {
    // Register with app manager
    appManager.registerPlugin('plugin_login');
    console.log('Plugin activated: plugin_login');
    
    // Get UI elements
    const statusContent = container.querySelector('#statusContent');
    const submitBtn = container.querySelector('#submitBtn');
    const clearBtn = container.querySelector('#clearBtn');
    const forgotPasswordLink = container.querySelector('#forgotPasswordLink');
    const signUpLink = container.querySelector('#signUpLink');
    
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
    
    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            // Clear the status content
            // Clear username and password fields
            const usernameField = container.querySelector('#inputUsername');
            const passwordField = container.querySelector('#inputPassword');
            if (usernameField) usernameField.value = '';
            if (passwordField) passwordField.value = '';
            if (statusContent) statusContent.innerHTML = '';
        });
    }

    if (submitBtn) {
        submitBtn.addEventListener('click', async () => {
            // Get values from input fields
            const usernameField = container.querySelector('#usernameField');
            const passwordField = container.querySelector('#passwordField');
            const username = usernameField ? usernameField.value : '';
            const password = passwordField ? passwordField.value : '';

            // Example: POST data to API
            try {
                const payload = { username, password };
                const response = await postData(payload);
                statusContent.innerHTML = `Login successful: ${response.message}`;
            } catch (error) {
                statusContent.innerHTML = `Login failed: ${error.message}`;
            }

            const published = appManager.publish('plugin_login', 'LoginCompleted', 
                { status: 'completed' }
            );
        });
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_login');
    };
}
