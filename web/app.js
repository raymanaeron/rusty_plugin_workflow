import { renderTermsPage } from './step-terms.js';
import { renderWiFiPage } from './step-wifi.js';

let step = 0;
export let network_connected = false;

export function goToNextStep() {
  step++;
  renderCurrentStep();
}

function renderCurrentStep() {
  const app = document.getElementById('app');
  app.innerHTML = '';
  if (step === 0) renderTermsPage(app);
  else if (step === 1) renderWiFiPage(app);
}

window.addEventListener('DOMContentLoaded', () => {
  renderCurrentStep();
});
