/**
 * JWT Manager for handling authentication tokens
 * Provides secure request functions and automatic token refresh
 */

let currentToken = null;
let tokenExpiry = null;
let refreshInterval = null;
let apiKey = null;
let apiSecret = null;
let sessionId = null;
let onTokenRefreshCallbacks = [];

/**
 * Initialize JWT authentication with API credentials
 * @param {string} key - API key
 * @param {string} secret - API secret
 * @returns {Promise<string>} - JWT token
 */
export async function initialize_with_jwt(key, secret) {
    apiKey = key || generateApiKey();
    apiSecret = secret || generateApiSecret();
    
    console.log('[JWT Manager] Initializing with API key:', apiKey);
    
    try {
        // Create a new session
        const session = await createSession(apiKey, apiSecret);
        sessionId = session.session_id;
        currentToken = session.token;
        
        // Parse token expiry (assuming JWT has an exp claim)
        tokenExpiry = getTokenExpiry(currentToken);
        
        // Set up automatic token refresh
        setupTokenRefresh();
        
        console.log('[JWT Manager] Successfully initialized');
        return currentToken;
    } catch (error) {
        console.error('[JWT Manager] Initialization failed:', error);
        throw error;
    }
}

/**
 * Create a secure request function that automatically adds the JWT token
 * @param {string} url - Request URL
 * @param {Object} options - Fetch options
 * @returns {Promise<Response>} - Fetch response
 */
export async function secure_request(url, options = {}) {
    // Ensure we have a valid token
    if (!currentToken) {
        throw new Error('JWT Manager not initialized. Call initialize_with_jwt() first');
    }

    // Check if token is about to expire and refresh if needed
    const now = Date.now() / 1000; // Current time in seconds
    if (tokenExpiry && (tokenExpiry - now < 60)) { // Refresh if less than 1 minute left
        await refreshToken();
    }
    
    // Create headers if not present
    const headers = options.headers || {};
    
    // Add authorization header
    headers['Authorization'] = `Bearer ${currentToken}`;
    
    // Return the fetch with the updated options
    return fetch(url, {
        ...options,
        headers
    });
}

/**
 * Register a callback to be called when token is refreshed
 * @param {Function} callback - Function to call with new token
 */
export function onTokenRefresh(callback) {
    if (typeof callback === 'function') {
        onTokenRefreshCallbacks.push(callback);
    }
}

/**
 * Create a new session with the API
 * @param {string} key - API key
 * @param {string} secret - API secret
 * @returns {Promise<Object>} - Session data including token
 */
async function createSession(key, secret) {
    try {
        const response = await fetch(`/api/auth/${key}/sessions`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ api_secret: secret }),
        });

        if (!response.ok) {
            throw new Error(`Failed to create session. Status: ${response.status}`);
        }

        return await response.json();
    } catch (error) {
        console.error('[JWT Manager] Error creating session:', error);
        throw error;
    }
}

/**
 * Get a new token for the current session
 * @returns {Promise<string>} - New JWT token
 */
async function refreshToken() {
    try {
        if (!apiKey || !sessionId) {
            throw new Error('Cannot refresh token: No active session');
        }

        console.log('[JWT Manager] Refreshing token...');
        
        const response = await fetch(`/api/auth/${apiKey}/sessions/${sessionId}`);
        
        if (!response.ok) {
            throw new Error(`Failed to refresh token. Status: ${response.status}`);
        }

        const data = await response.json();
        currentToken = data.token;
        tokenExpiry = getTokenExpiry(currentToken);
        
        // Notify all registered callbacks
        onTokenRefreshCallbacks.forEach(callback => {
            try {
                callback(currentToken);
            } catch (e) {
                console.error('[JWT Manager] Error in token refresh callback:', e);
            }
        });
        
        console.log('[JWT Manager] Token refreshed successfully');
        return currentToken;
    } catch (error) {
        console.error('[JWT Manager] Error refreshing token:', error);
        throw error;
    }
}

/**
 * Set up automatic token refresh every minute
 */
function setupTokenRefresh() {
    // Clear existing interval if any
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
    
    // Refresh token every minute
    refreshInterval = setInterval(async () => {
        try {
            await refreshToken();
        } catch (error) {
            console.error('[JWT Manager] Scheduled token refresh failed:', error);
        }
    }, 60000); // 60 seconds
}

/**
 * Extract expiry time from JWT token
 * @param {string} token - JWT token
 * @returns {number|null} - Expiry time in seconds or null if not present
 */
function getTokenExpiry(token) {
    try {
        // Extract the payload part (second segment) of the JWT
        const payload = token.split('.')[1];
        // Base64 decode and parse as JSON
        const decoded = JSON.parse(atob(payload));
        return decoded.exp || null;
    } catch (error) {
        console.error('[JWT Manager] Error parsing token expiry:', error);
        return null;
    }
}

/**
 * Clean up resources and clear refresh interval
 */
export function cleanup() {
    if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = null;
    }
    
    // Revoke the session if we have active credentials
    if (apiKey && sessionId) {
        try {
            fetch(`/api/auth/${apiKey}/sessions/${sessionId}`, {
                method: 'DELETE',
            }).catch(e => console.error('[JWT Manager] Error revoking session:', e));
        } catch (e) {
            // Ignore errors during cleanup
        }
    }
    
    currentToken = null;
    tokenExpiry = null;
    apiKey = null;
    apiSecret = null;
    sessionId = null;
    onTokenRefreshCallbacks = [];
}

/**
 * Generate a random API key if not provided
 * @returns {string} - Random API key
 */
function generateApiKey() {
    return generateRandomString(12);
}

/**
 * Generate a random API secret if not provided
 * @returns {string} - Random API secret
 */
function generateApiSecret() {
    return generateRandomString(24);
}

/**
 * Generate a random string of specified length
 * @param {number} length - Length of string to generate
 * @returns {string} - Random string
 */
function generateRandomString(length = 12) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

// Handle page unload to clean up resources
if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', cleanup);
}
