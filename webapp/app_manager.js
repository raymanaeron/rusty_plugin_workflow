class AppManager {
    constructor() {
        this.ws = null;
        this.isConnecting = false;
        this.reconnectTimeout = null;
        this.subscribers = new Map();
        this.activePlugins = new Set();
        this.ready = false;
        this.readyCallbacks = [];
        this.connect();
    }

    onReady(callback) {
        if (this.ready) {
            callback();
        } else {
            this.readyCallbacks.push(callback);
        }
    }

    setReady() {
        this.ready = true;
        this.readyCallbacks.forEach(callback => callback());
        this.readyCallbacks = [];
    }

    connect() {
        if (this.isConnecting || (this.ws?.readyState === WebSocket.OPEN)) {
            console.log('[appManager] Connection already exists or in progress');
            return;
        }

        this.cleanup();
        this.isConnecting = true;
        console.log('[appManager] Initiating new connection');

        this.ws = new WebSocket('ws://localhost:8081/ws');

        this.ws.onopen = () => {
            this.isConnecting = false;
            console.log('[appManager] Connected successfully');
            this.ws.send('register-name:webapp_manager');
            // Resubscribe to all topics
            for (const topic of this.subscribers.keys()) {
                this.ws.send(`subscribe:${topic}`);
            }
            this.setReady(); // Move setReady() here
        };

        this.ws.onclose = () => {
            console.log('[appManager] Connection closed');
            this.scheduleReconnect();
        };

        this.ws.onerror = (error) => {
            console.error('[appManager] Connection error:', error);
            this.scheduleReconnect();
        };

        this.ws.onmessage = (event) => this.handleMessage(event);
    }

    scheduleReconnect() {
        this.isConnecting = false;
        if (!this.reconnectTimeout) {
            this.reconnectTimeout = setTimeout(() => {
                this.reconnectTimeout = null;
                this.connect();
            }, 2000);
        }
    }

    cleanup() {
        if (this.ws) {
            this.ws.onclose = null;
            this.ws.close();
            this.ws = null;
        }
    }

    registerPlugin(pluginName) {
        this.activePlugins.add(pluginName);
        console.log(`[appManager] Plugin registered: ${pluginName}`);
    }

    unregisterPlugin(pluginName) {
        this.activePlugins.delete(pluginName);
        console.log(`[appManager] Plugin unregistered: ${pluginName}`);
    }

    subscribe(plugin, topic, callback) {
        if (!this.subscribers.has(topic)) {
            this.subscribers.set(topic, new Map());
            if (this.ws?.readyState === WebSocket.OPEN) {
                this.ws.send(`subscribe:${topic}`);
            }
        }
        this.subscribers.get(topic).set(plugin, callback);
        console.log(`[appManager] ${plugin} subscribed to ${topic}`);
    }

    unsubscribe(plugin, topic) {
        const topicSubscribers = this.subscribers.get(topic);
        if (topicSubscribers) {
            topicSubscribers.delete(plugin);
            if (topicSubscribers.size === 0) {
                this.subscribers.delete(topic);
                if (this.ws?.readyState === WebSocket.OPEN) {
                    this.ws.send(`unsubscribe:${topic}`);
                }
            }
        }
    }

    publish(plugin, topic, payload) {
        if (this.ws?.readyState !== WebSocket.OPEN) {
            console.warn(`[appManager] Cannot publish from ${plugin}: connection not open`);
            return false;
        }

        const message = {
            publisher_name: plugin,
            topic: topic,
            payload: JSON.stringify(payload),
            timestamp: new Date().toISOString()
        };

        this.ws.send(`publish-json:${JSON.stringify(message)}`);
        console.log(`[appManager] ${plugin} published to ${topic}`);
        return true;
    }

    handleMessage(event) {
        console.log('[appManager] Raw message received:', event.data);
        try {
            // Log raw message for debugging
            console.log('[appManager] Raw message received:', event.data);

            let messageData = event.data;

            // Check if the message is a prefixed JSON message
            if (typeof messageData === 'string' && messageData.startsWith('publish-json:')) {
                messageData = messageData.substring('publish-json:'.length);
            }

            const data = JSON.parse(messageData);
            console.log('[appManager] Parsed message:', data);
            console.log('[appManager] Received topic:', data.topic);

            const topic = data.topic;
            const subscribers = this.subscribers.get(topic);

            if (subscribers) {
                subscribers.forEach((callback, plugin) => {
                    console.log(`[appManager] Delivering ${topic} message to ${plugin}`);

                    try {
                        // If payload is a JSON string, parse it
                        if (typeof data.payload === 'string') {
                            try {
                                data.payload = JSON.parse(data.payload);
                            } catch (e) {
                                // If it can't be parsed as JSON, keep it as a string
                                console.log(`[appManager] Payload is not JSON, using as string: ${data.payload}`);
                            }
                        }

                        callback(data);
                    } catch (callbackErr) {
                        console.error(`[appManager] Error in ${plugin}'s callback:`, callbackErr);
                    }
                });
            } else {
                console.log(`[appManager] No subscribers for topic: ${topic}`);
            }
        } catch (err) {
            console.error('[appManager] Error processing message:', err, 'Raw data:', event.data);
        }
    }

    hasSubscribers(topic) { // Added this method
        return this.subscribers.has(topic);
    }
}

export const appManager = new AppManager();