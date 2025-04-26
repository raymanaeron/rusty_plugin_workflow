class WebSocketConnectionManager {
    constructor() {
        this.ws = null;
        this.isConnecting = false;
        this.reconnectTimeout = null;
        this.subscribers = new Map();
        this.activePlugins = new Set();
        this.connect();
    }

    connect() {
        if (this.isConnecting || (this.ws?.readyState === WebSocket.OPEN)) {
            console.log('[WSManager] Connection already exists or in progress');
            return;
        }

        this.cleanup();
        this.isConnecting = true;
        console.log('[WSManager] Initiating new connection');
        
        this.ws = new WebSocket('ws://localhost:8081/ws');

        this.ws.onopen = () => {
            this.isConnecting = false;
            console.log('[WSManager] Connected successfully');
            this.ws.send('register-name:webapp_manager');
            // Resubscribe to all topics
            this.subscribers.forEach((_, topic) => {
                this.ws.send(`subscribe:${topic}`);
            });
        };

        this.ws.onclose = () => {
            console.log('[WSManager] Connection closed');
            this.scheduleReconnect();
        };

        this.ws.onerror = (error) => {
            console.error('[WSManager] Connection error:', error);
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
        console.log(`[WSManager] Plugin registered: ${pluginName}`);
    }

    unregisterPlugin(pluginName) {
        this.activePlugins.delete(pluginName);
        console.log(`[WSManager] Plugin unregistered: ${pluginName}`);
    }

    subscribe(plugin, topic, callback) {
        if (!this.subscribers.has(topic)) {
            this.subscribers.set(topic, new Map());
            if (this.ws?.readyState === WebSocket.OPEN) {
                this.ws.send(`subscribe:${topic}`);
            }
        }
        this.subscribers.get(topic).set(plugin, callback);
        console.log(`[WSManager] ${plugin} subscribed to ${topic}`);
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
            console.warn(`[WSManager] Cannot publish from ${plugin}: connection not open`);
            return false;
        }

        const message = {
            publisher_name: plugin,
            topic: topic,
            payload: JSON.stringify(payload),
            timestamp: new Date().toISOString()
        };

        this.ws.send(`publish-json:${JSON.stringify(message)}`);
        console.log(`[WSManager] ${plugin} published to ${topic}`);
        return true;
    }

    handleMessage(event) {
        try {
            const data = JSON.parse(event.data);
            const subscribers = this.subscribers.get(data.topic);
            if (subscribers) {
                subscribers.forEach((callback, plugin) => {
                    console.log(`[WSManager] Delivering ${data.topic} message to ${plugin}`);
                    callback(data);
                });
            }
        } catch (err) {
            console.error('[WSManager] Error processing message:', err);
        }
    }
}

export const wsManager = new WebSocketConnectionManager();
