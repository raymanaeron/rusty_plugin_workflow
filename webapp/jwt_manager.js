/**
 * JWT Manager for handling authentication tokens
 * Provides secure request functions and automatic token refresh
 */

// Session state management
let apiKey = null;
let apiSecret = null;
let sessionId = null;
let currentToken = null;

/**
 * Initialize JWT authentication with API credentials
 * @param {string} key - API key (optional, will generate if not provided)
 * @param {string} secret - API secret (optional, will generate if not provided)
 * @returns {Promise<string>} - JWT token
 */
export async function initialize_with_jwt(key, secret) {
    // Generate or use provided credentials
    apiKey = key || generateRandomString(12);
    apiSecret = secret || generateRandomString(24);
    
    console.log('[JWT Manager] Initializing with API key:', apiKey);
    
    try {
        // Create a new session
        const session = await createSession(apiKey, apiSecret);
        
        // Store session data
        sessionId = session.session_id;
        currentToken = session.token;
        
        console.log('[JWT Manager] Session created successfully:', sessionId);
        console.log('[JWT Manager] Token obtained:', currentToken ? '✓' : '✗');
        
        return currentToken;
    } catch (error) {
        console.error('[JWT Manager] Initialization failed:', error);
        throw error;
    }
}

/**
 * Make a secure request with authentication token
 * @param {string} url - Request URL
 * @param {Object} options - Fetch options
 * @returns {Promise<Response>} - Fetch response
 */
export async function secure_request(url, options = {}) {
    if (!currentToken) {
        throw new Error('[JWT Manager] No token available. Call initialize_with_jwt() first');
    }

    // Create headers if not present
    const headers = options.headers || {};
    
    // Add authorization header with current token
    headers['Authorization'] = `Bearer ${currentToken}`;
    
    try {
        const response = await fetch(url, {
            ...options,
            headers
        });
        
        // If unauthorized, try refreshing token and retry once
        if (response.status === 401) {
            console.log('[JWT Manager] Received 401 unauthorized, refreshing token');
            
            try {
                // Get a fresh token for the current session
                const refreshResponse = await getSessionToken(apiKey, sessionId);
                if (refreshResponse && refreshResponse.token) {
                    // Update the token
                    currentToken = refreshResponse.token;
                    
                    // Retry the request with new token
                    headers['Authorization'] = `Bearer ${currentToken}`;
                    return fetch(url, { ...options, headers });
                }
            } catch (refreshError) {
                console.error('[JWT Manager] Token refresh failed:', refreshError);
            }
        }
        
        return response;
    } catch (error) {
        console.error('[JWT Manager] Request failed:', error);
        throw error;
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
            const errorText = await response.text();
            throw new Error(`Failed to create session (${response.status}): ${errorText}`);
        }

        return await response.json();
    } catch (error) {
        console.error('[JWT Manager] Error creating session:', error);
        throw error;
    }
}

/**
 * Get a new token for a specific session
 * @param {string} key - API key
 * @param {string} session - Session ID
 * @returns {Promise<Object>} - Token data
 */
async function getSessionToken(key, session) {
    try {
        const response = await fetch(`/api/auth/${key}/sessions/${session}`);
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`Failed to get session token (${response.status}): ${errorText}`);
        }

        return await response.json();
    } catch (error) {
        console.error('[JWT Manager] Error getting session token:', error);
        throw error;
    }
}

/**
 * Clean up resources and revoke the session
 */
export function cleanup() {
    if (apiKey && sessionId) {
        // Attempt to revoke the session
        try {
            fetch(`/api/auth/${apiKey}/sessions/${sessionId}`, {
                method: 'DELETE',
            }).catch(e => console.error('[JWT Manager] Error revoking session:', e));
        } catch (e) {
            // Ignore errors during cleanup
        }
    }
    
    // Reset state
    apiKey = null;
    apiSecret = null;
    sessionId = null;
    currentToken = null;
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
