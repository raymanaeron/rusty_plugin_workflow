import { AuthManager } from './auth-manager.js';

export async function activate(container, appManager) {
    // Register with app manager
    appManager.registerPlugin('plugin_login');
    console.log('Plugin activated: plugin_login');
    
    // Amazon LWA Configuration
    const AMAZON_CLIENT_ID = 'amzn1.application-oa2-client.yourClientIdHere';
    const REDIRECT_URI = window.location.origin + '/login/web/callback';
    
    // State management
    let currentView = 'initialView';
    
    // Initialize auth manager
    const authManager = new AuthManager(AMAZON_CLIENT_ID, REDIRECT_URI);
    
    // Get UI elements
    const loginContainer = container.querySelector('#loginContainer');
    const resultBox = container.querySelector('#resultBox');
    const amazonLoginButton = container.querySelector('#amazon-login-button');
    const skipLoginBtn = container.querySelector('#skipLoginBtn');
    const continueGuestBtn = container.querySelector('#continueGuestBtn');
    const continueAfterLoginBtn = container.querySelector('#continueAfterLoginBtn');
    const tryAgainBtn = container.querySelector('#tryAgainBtn');
    const userNameDisplay = container.querySelector('#userNameDisplay');
    const errorMessage = container.querySelector('#errorMessage');
    
    // Helper function to show a specific view
    function showView(viewName) {
        currentView = viewName;
        
        // Hide all views
        document.querySelectorAll('.login-view').forEach(view => {
            view.style.display = 'none';
        });
        
        // Show the requested view
        const viewToShow = document.getElementById(viewName);
        if (viewToShow) {
            viewToShow.style.display = 'block';
        }
    }
    
    // Initialize Amazon Login button
    authManager.renderLoginButton(
        amazonLoginButton,
        handleLoginSuccess,
        handleLoginError
    );
    
    // Handle successful login
    async function handleLoginSuccess(profile) {
        console.log('[plugin_login] Login successful', profile);
        
        // Display user info
        userNameDisplay.textContent = profile.name || 'User';
        
        // Show success view
        showView('successView');
        
        // Optionally store user data locally
        localStorage.setItem('userProfile', JSON.stringify(profile));
    }
    
    // Handle login error
    function handleLoginError(error) {
        console.error('[plugin_login] Login error', error);
        
        // Display error message
        errorMessage.textContent = 'Authentication failed. Please try again.';
        
        // Show error view
        showView('errorView');
    }
    
    // Handle API calls
    async function validateUserSession() {
        try {
            showView('loadingView');
            
            const response = await fetch('/api/login/user/session', {
                method: 'GET',
                headers: { 'Content-Type': 'application/json' }
            });
            
            if (response.ok) {
                const data = await response.json();
                return data;
            } else {
                throw new Error('Session validation failed');
            }
        } catch (error) {
            console.error('[plugin_login] Session validation error:', error);
            throw error;
        }
    }
    
    // Event Handlers
    if (skipLoginBtn) {
        skipLoginBtn.addEventListener('click', () => {
            // Publish skip event
            const published = appManager.publish('plugin_login', 'UserCompleted', 
                { status: 'skipped' }
            );
            
            if (published) {
                console.log("[plugin_login] Login skip published");
                resultBox.innerHTML = '<div class="alert alert-info">Login skipped. Redirecting...</div>';
            } else {
                console.warn("[plugin_login] Skip publish failed");
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to skip login</div>';
            }
        });
    }
    
    if (continueGuestBtn) {
        continueGuestBtn.addEventListener('click', async () => {
            // Continue as guest
            const published = appManager.publish('plugin_login', 'UserCompleted', 
                { status: 'guest' }
            );
            
            if (published) {
                console.log("[plugin_login] Continuing as guest");
                resultBox.innerHTML = '<div class="alert alert-success">Continuing as guest. Redirecting...</div>';
            } else {
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to continue</div>';
            }
        });
    }
    
    if (continueAfterLoginBtn) {
        continueAfterLoginBtn.addEventListener('click', async () => {
            // Publish completion with user data
            const userProfile = authManager.profile || {};
            const published = appManager.publish('plugin_login', 'UserCompleted', 
                { 
                    status: 'completed',
                    userId: userProfile.user_id,
                    userEmail: userProfile.email,
                    userName: userProfile.name
                }
            );
            
            if (published) {
                console.log("[plugin_login] Login completion published");
                resultBox.innerHTML = '<div class="alert alert-success">Login successful! Redirecting...</div>';
            } else {
                console.warn("[plugin_login] Login completion publish failed");
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to complete login</div>';
            }
        });
    }
    
    if (tryAgainBtn) {
        tryAgainBtn.addEventListener('click', () => {
            // Reset to initial view
            showView('initialView');
        });
    }
    
    // Check for existing session on load
    try {
        const sessionData = await validateUserSession();
        if (sessionData && sessionData.authenticated) {
            // Auto-login with existing session
            userNameDisplay.textContent = sessionData.userName || 'User';
            showView('successView');
        } else {
            // Show initial login view
            showView('initialView');
        }
    } catch (error) {
        // Default to initial view on error
        showView('initialView');
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_login');
    };
}