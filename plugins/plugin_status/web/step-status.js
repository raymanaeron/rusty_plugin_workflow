const statusContent = document.getElementById('statusContent');
const wsUrl = `ws://${location.host}/status/ws`;
let socket;

function connectWebSocket() {
  socket = new WebSocket(wsUrl);

  socket.onopen = () => {
    console.log('[status] WebSocket connected');
  };

  socket.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data);
      if (message.action === 'status' && typeof message.data?.text === 'string') {
        statusContent.textContent = message.data.text;
      }
    } catch (err) {
      console.error('[status] Failed to parse message', err);
    }
  };

  socket.onclose = () => {
    console.warn('[status] WebSocket closed. Reconnecting...');
    setTimeout(connectWebSocket, 1000);
  };
}

connectWebSocket();
