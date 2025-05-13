export async function activate(container, appManager, jwtManager) {
    appManager.registerPlugin('plugin_provisioning');
    console.log('Plugin activated: plugin_provisioning');
    const statusContent = container.querySelector('#statusContent');
    const continueBtn = container.querySelector('#continueBtn');
    const spinner = container.querySelector('.loading-spinner'); // Updated selector for daisyUI spinner

    let doneIcon = null;

    // Status messages to display
    const messages = [
        "Authenticating device identity..",
        "Device authentication successful..",
        "Linking device to your account..",
        "Device successfully added to your account..",
        "Applying default settings for your device..",
        "Default settings configured..",
        "Fetching product-specific attributes..",
        "Product attributes successfully applied..",
        "Provisioning complete..",
        "Press the [Continue] button to proceed.."
    ];

    let step = 0;
    function updateStatus() {
        if (statusContent) statusContent.textContent = messages[step];
        if (step === messages.length - 1) {
            // Last message: enable button and hide spinner
            if (continueBtn) continueBtn.disabled = false;
            if (spinner) spinner.style.display = "none";
            // Show done icon in place of spinner
            if (!doneIcon) {
                doneIcon = document.createElement('img');
                doneIcon.src = '/execution/web/icons/exec-plan-done.svg';
                doneIcon.alt = 'Device Provisioning Done';
                doneIcon.style.width = '3rem';
                doneIcon.style.height = '3rem';
                // Insert the icon where the spinner was
                spinner.parentNode.insertBefore(doneIcon, spinner);
            }
            doneIcon.style.display = "";
        } else {
            // Spinner should be visible and button disabled while messages are updating
            if (spinner) spinner.style.display = "";
            if (continueBtn) continueBtn.disabled = true;
            // Hide done icon if present
            if (doneIcon) doneIcon.style.display = "none";
            setTimeout(() => {
                step++;
                updateStatus();
            }, 300);
        }
    }
    updateStatus();

    // Continue button click handler
    if (continueBtn) {
        continueBtn.addEventListener('click', () => {
            // Publish completion event
            const published = appManager.publish('plugin_provisioning', 'ProvisionCompleted', {
                status: 'completed'
            });

            if (published) {
                console.log("[plugin_provisioning] Completion status published");
                statusContent.innerHTML = '<div class="alert alert-success">Setup complete! Redirecting...</div>';
                continueBtn.disabled = true;
            } else {
                console.warn("[plugin_provisioning] Completion publish failed");
                statusContent.innerHTML = '<div class="alert alert-warning">Failed to complete. Please try again.</div>';
            }
        });
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_provisioning');
    };
}