const statusContent = document.getElementById('statusContent');

function pollStatus() {
  fetch('/api/{{plugin_route}}/{{resource_name}}', { method: 'GET' })
    .then(response => response.json())
    .then(message => {
      console.log('[{{plugin_route}}] API response:', message);
      if (typeof message.status === 'string') {
        statusContent.textContent = message.status;
      }
    })
    .catch(err => console.error('[{{plugin_route}}] Failed to fetch status', err))
    .finally(() => setTimeout(pollStatus, 1000));
}

pollStatus();