const statusContent = document.getElementById('statusContent');

function pollStatus() {
  fetch('/api/settings/deviceconfigs', { method: 'GET' })
    .then(response => response.json())
    .then(message => {
      console.log('[settings] API response:', message);
      if (typeof message.status === 'string') {
        statusContent.textContent = message.status;
      }
    })
    .catch(err => console.error('[settings] Failed to fetch status', err))
    .finally(() => setTimeout(pollStatus, 1000));
}

pollStatus();
