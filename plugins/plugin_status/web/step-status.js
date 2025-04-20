const statusContent = document.getElementById('statusContent');

function pollStatus() {
  fetch('/api/status/statusmessage', { method: 'GET' })
    .then(response => response.json())
    .then(message => {
      console.log('[status] API response:', message);  //  MUST see this in console

      if (typeof message.status === 'string') {
        statusContent.textContent = message.status;
      }

    })
    .catch(err => console.error('[status] Failed to fetch status', err))
    .finally(() => setTimeout(pollStatus, 1000));
}

// our web app is vanilla JS, and therefore no two way binding possible without a major workaround
// websocket is not possible because that will require us to host a web server on the rust side
// for now polling just works fine, and is the easiest solution
pollStatus();
