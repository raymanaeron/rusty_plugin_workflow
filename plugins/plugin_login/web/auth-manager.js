/**
 * Amazon Login with Amazon (LWA) Auth Manager
 * Handles authentication flow with Amazon's Login service
 */

export class AuthManager {
  constructor(clientId, redirectUri, scopes = ['profile', 'email']) {
    this.clientId = clientId;
    this.redirectUri = redirectUri;
    this.scopes = scopes;
    this.accessToken = null;
    this.profile = null;
    this.authState = 'idle'; // idle, loading, authenticated, error
    this.amazonLoginReady = false;
    
    // Initialize Amazon SDK when it's ready
    if (window.amazon) {
      this.initAmazonLogin();
    } else {
      document.addEventListener('amazon-sdk-loaded', () => {
        this.initAmazonLogin();
      });
      
      // Fallback in case event isn't triggered
      window.onAmazonLoginReady = () => {
        this.initAmazonLogin();
      };
    }
  }
  
  /**
   * Initialize the Amazon SDK with our client id
   */
  initAmazonLogin() {
    if (window.amazon && window.amazon.Login) {
      window.amazon.Login.setClientId(this.clientId);
      this.amazonLoginReady = true;
      console.log('[AuthManager] Amazon Login SDK initialized');
    } else {
      console.error('[AuthManager] Amazon Login SDK not available');
    }
  }
  
  /**
   * Renders the Amazon login button in the specified container
   * @param {HTMLElement} container - The DOM element to render the button in
   * @param {Function} onAuthSuccess - Callback for successful authentication
   * @param {Function} onAuthError - Callback for authentication failures
   */
  renderLoginButton(container, onAuthSuccess, onAuthError) {
    if (!this.amazonLoginReady) {
      const checkInterval = setInterval(() => {
        if (this.amazonLoginReady) {
          clearInterval(checkInterval);
          this.renderLoginButton(container, onAuthSuccess, onAuthError);
        }
      }, 500);
      return;
    }
    
    container.innerHTML = '';
    
    window.amazon.Login.renderButton(
      container,
      { 
        type: 'standard',
        size: 'medium', 
        color: 'gold', 
        language: 'en-US',
        authorization: {
          scope: this.scopes.join(' '),
          responseType: 'token'
        },
      },
      (response) => this.handleAuthResponse(response, onAuthSuccess, onAuthError)
    );
  }
  
  /**
   * Handle the authentication response from Amazon
   */
  async handleAuthResponse(response, onSuccess, onError) {
    try {
      if (response.error) {
        this.authState = 'error';
        console.error('[AuthManager] Authentication error:', response.error);
        if (onError) onError(response.error);
        return;
      }
      
      // Store the access token
      this.accessToken = response.access_token;
      this.authState = 'loading';
      
      // Fetch the user profile
      await this.fetchUserProfile();
      
      // Validate token with our backend
      await this.validateTokenWithBackend();
      
      // Mark as authenticated
      this.authState = 'authenticated';
      if (onSuccess) onSuccess(this.profile);
      
    } catch (error) {
      this.authState = 'error';
      console.error('[AuthManager] Error during authentication:', error);
      if (onError) onError(error);
    }
  }
  
  /**
   * Fetch the user profile from Amazon
   */
  async fetchUserProfile() {
    return new Promise((resolve, reject) => {
      if (!this.accessToken) {
        reject(new Error('No access token available'));
        return;
      }
      
      window.amazon.Login.retrieveProfile(this.accessToken, (response) => {
        if (response.success) {
          this.profile = response.profile;
          resolve(this.profile);
        } else {
          reject(new Error('Failed to retrieve profile'));
        }
      });
    });
  }
  
  /**
   * Validate the Amazon token with our backend
   */
  async validateTokenWithBackend() {
    try {
      const response = await fetch('/api/login/user/validate-amazon', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          accessToken: this.accessToken,
          userProfile: this.profile
        })
      });
      
      if (!response.ok) {
        throw new Error('Token validation failed');
      }
      
      const data = await response.json();
      return data;
    } catch (error) {
      console.error('[AuthManager] Token validation error:', error);
      throw error;
    }
  }
  
  /**
   * Check if the user is authenticated
   */
  isAuthenticated() {
    return this.authState === 'authenticated' && this.accessToken !== null;
  }
  
  /**
   * Sign out the current user
   */
  signOut() {
    if (window.amazon && window.amazon.Login) {
      window.amazon.Login.logout();
    }
    
    this.accessToken = null;
    this.profile = null;
    this.authState = 'idle';
  }
}

// Listen for the Amazon SDK loaded event
document.addEventListener('DOMContentLoaded', () => {
  if (window.amazon && window.amazon.Login) {
    document.dispatchEvent(new Event('amazon-sdk-loaded'));
  }
});

window.onAmazonLoginReady = function() {
  document.dispatchEvent(new Event('amazon-sdk-loaded'));
};
