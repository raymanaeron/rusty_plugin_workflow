export async function activate(container, appManager, jwtManager) {
    // Register with connection manager
    appManager.registerPlugin('plugin_login');
    
    // Get DOM elements
    const usernameInput = container.querySelector('#username');
    const passwordInput = container.querySelector('#password');
    const rememberCheck = container.querySelector('#rememberCheck');
    const loginBtn = container.querySelector('#loginBtn');
    const resultBox = container.querySelector('#resultBox');
    
    // Set focus to username field
    if (usernameInput) {
        setTimeout(() => {
            usernameInput.focus();
        }, 100);
    }
    
    // Handle login button click
    if (loginBtn) {
        loginBtn.addEventListener('click', async () => {
            // Basic validation
            const username = usernameInput?.value?.trim();
            const password = passwordInput?.value;
            const rememberMe = rememberCheck?.checked || false;
            
            if (!username || !password) {
                resultBox.innerHTML = `<div class="alert alert-warning">Please enter both username and password.</div>`;
                return;
            }
            
            // Show loading state
            loginBtn.disabled = true;
            loginBtn.innerHTML = `<span class="loading loading-spinner loading-sm"></span> Logging in...`;
            resultBox.innerHTML = "";
            
            try {
                // Call login API
                const response = await jwtManager.secure_request('/api/login/userprofile', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        username,
                        password,
                        remember_me: rememberMe
                    })
                });
                
                console.log("[plugin_login] Login response:", response);

                const data = await response.json();
                
                console.log("[plugin_login] JSON response:", data);

                if (response.ok) {
                    // Login successful
                    resultBox.innerHTML = `<div class="alert alert-success">Login successful! Redirecting...</div>`;
                    
                    // Publish login event
                    const published = appManager.publish('plugin_login', 'LoginCompleted', {
                        username: username,
                        userId: data.user_id || 'unknown',
                        isAdmin: data.is_admin || false
                    });
                    
                    if (published) {
                        console.log("[plugin_login] Authentication published");
                    } else {
                        console.warn("[plugin_login] Authentication publish failed");
                    }
                    
                    // Simulate delay before redirect
                    /*
                    setTimeout(() => {
                        // Get next route from response or use default
                        const nextRoute = data.next_route || "/dashboard";
                        history.pushState({}, "", nextRoute);
                        window.dispatchEvent(new PopStateEvent("popstate"));
                    }, 1500);
                    */
                } else {
                    // Login failed
                    resultBox.innerHTML = `<div class="alert alert-error">${data.message || "Login failed."}</div>`;
                }
            } catch (err) {
                resultBox.innerHTML = `<div class="alert alert-error">Error: ${err.message}</div>`;
                console.error(err);
            } finally {
                // Reset button state
                loginBtn.disabled = false;
                loginBtn.textContent = "Login";
            }
        });
    }

    // Return cleanup function at module level
    // Unregisters the plugin from the application manager
    return () => {
        appManager.unregisterPlugin('plugin_login');
    };
}
