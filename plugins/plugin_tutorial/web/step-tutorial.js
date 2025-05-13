export async function activate(container, appManager, jwtManager) {
    // Register this plugin with the application manager for lifecycle management
    appManager.registerPlugin('plugin_tutorial');
    console.log('Plugin activated: plugin_tutorial');
    
    // Find and cache DOM elements for UI interaction
    const statusContent = container.querySelector('#statusContent');
    const actionBtn = container.querySelector('#actionBtn');
    const skipBtn = container.querySelector('#skipBtn');
    const continueBtn = container.querySelector('#continueBtn');
    const resultBox = container.querySelector('#resultBox');
    
    // YouTube player elements
    const playerElement = container.querySelector('#player');
    const playPauseBtn = container.querySelector('#playPauseBtn');
    const timeDisplay = container.querySelector('#timeDisplay');
    const progressBar = container.querySelector('#progressBar');
    const progressContainer = container.querySelector('#progressContainer');
    
    // Initialize YouTube player functionality
    let player;
    let updateInterval;
    
    // Create YouTube Player API script
    const tag = document.createElement('script');
    tag.src = "https://www.youtube.com/iframe_api";
    const firstScriptTag = document.getElementsByTagName('script')[0];
    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
    
    // Setup global player functions
    window.tutorialPlayer = {
        togglePlay: () => {
            if (!player) return;
            const state = player.getPlayerState();
            if (state == YT.PlayerState.PLAYING) {
                player.pauseVideo();
            } else {
                player.playVideo();
            }
        },
        rewind10: () => {
            if (!player) return;
            const currentTime = player.getCurrentTime();
            player.seekTo(Math.max(currentTime - 10, 0), true);
        },
        forward10: () => {
            if (!player) return;
            const currentTime = player.getCurrentTime();
            player.seekTo(currentTime + 10, true);
        }
    };
    
    // Format time for display
    function formatTime(seconds) {
        seconds = Math.floor(seconds);
        const minutes = Math.floor(seconds / 60);
        let secs = seconds % 60;
        if (secs < 10) secs = "0" + secs;
        return minutes + ":" + secs;
    }
    
    // Update progress bar and time display
    function updateProgress() {
        if (player && player.getDuration) {
            const currentTime = player.getCurrentTime();
            const duration = player.getDuration();
            const percentage = (currentTime / duration) * 100;
            progressBar.style.width = percentage + "%";
            timeDisplay.textContent = formatTime(currentTime) + " / " + formatTime(duration);
        }
    }
    
    // Setup event handler for seeking in video
    if (progressContainer) {
        progressContainer.addEventListener('click', (event) => {
            if (!player) return;
            
            const rect = progressContainer.getBoundingClientRect();
            const clickX = event.clientX - rect.left;
            const width = rect.width;
            const percentage = clickX / width;
            const duration = player.getDuration();
            player.seekTo(percentage * duration, true);
        });
    }
    
    // Initialize YouTube Player when API is ready
    window.onYouTubeIframeAPIReady = () => {
        player = new YT.Player(playerElement, {
            height: '270',
            width: '100%',
            videoId: 'KBs63IWzS6E', // You can change this to any YouTube video ID
            playerVars: {
                controls: 0,
                modestbranding: 1,
                rel: 0,
                fs: 0,
                iv_load_policy: 3,
                disablekb: 1,
                playsinline: 1
            },
            events: {
                'onReady': () => {
                    updateInterval = setInterval(updateProgress, 1000);
                },
                'onStateChange': (event) => {
                    if (event.data == YT.PlayerState.PLAYING) {
                        playPauseBtn.textContent = '⏸️ Pause';
                    } else {
                        playPauseBtn.textContent = '▶️ Play';
                    }
                }
            }
        });
    };
    
    // Fetch all resources from the plugin's REST API endpoint
    // Returns a promise with the retrieved data or throws an error
    async function getData() {
        try {
            const response = await fetch('/api/tutorial/tutcontent');
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
            const response = await fetch('/api/tutorial/tutcontent', {
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
            const response = await fetch(`/api/tutorial/tutcontent/${id}`, {
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
            const response = await fetch(`/api/tutorial/tutcontent/${id}`, {
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
            const published = appManager.publish('plugin_tutorial', 'TutorialCompleted', 
                { status: 'skipped' }
            );
            
            if (published) {
                console.log("[plugin_tutorial] Skip status published");
                resultBox.innerHTML = '<div class="alert alert-info">Setup skipped. Redirecting...</div>';
            } else {
                console.warn("[plugin_tutorial] Skip publish failed");
                resultBox.innerHTML = '<div class="alert alert-warning">Failed to publish skip status</div>';
            }
        });
    }
    
    // Handle the continue button click event
    // Publishes a completion status event
    if (continueBtn) {
        continueBtn.addEventListener('click', async () => {
            // Publish completion event 
            const published = appManager.publish('plugin_tutorial', 'TutorialCompleted', 
                { status: 'completed' }
            );
            
            if (published) {
                console.log("[plugin_tutorial] Completion status published");
            } else {
                console.warn("[plugin_tutorial] Completion publish failed");
            }
        });
    }

    // Return cleanup function at module level
    // Unregisters the plugin from the application manager
    return () => {
        // Clear the progress update interval when plugin is deactivated
        if (updateInterval) {
            clearInterval(updateInterval);
        }
        appManager.unregisterPlugin('plugin_tutorial');
    };
}
