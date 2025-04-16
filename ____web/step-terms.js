import { goToNextStep } from './app.js';

export let network_connected = false;

export function renderTermsPage(container) {
  const section = document.createElement('section');
  section.innerHTML = `
    <h1>Terms and Conditions</h1>
    <p>By proceeding, you agree to the device's terms and conditions of use.</p>
    <button id="accept-btn">Accept</button>
  `;

  container.appendChild(section);

  document.getElementById('accept-btn').addEventListener('click', () => {
    goToNextStep();
  });
}
