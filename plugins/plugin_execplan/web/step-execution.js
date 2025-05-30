export async function activate(container, appManager, jwtManager) {
    appManager.registerPlugin('plugin_execplan');
    console.log('Plugin activated: plugin_execplan');
    const statusContent = container.querySelector('#statusContent');
    const continueBtn = container.querySelector('#continueBtn');
    const spinnerContainer = container.querySelector('#loadingSpinner');
    const spinner = spinnerContainer.querySelector('.loading-spinner');

    let doneIcon = null;

    // Status messages to display
    const messages = [
        "Checking for updated execution plan..",
        "Execution plan update required..",
        "Downloading new execution plan and related plugins..",
        "Download completed..",
        "Applying new execution plan..",
        "New execution plan is ready..",
        "Press the [Continue] button to proceed.."
    ];

    let step = 0;
    function updateStatus() {
        if (statusContent) statusContent.textContent = messages[step];
        if (step === messages.length - 1) {
            // Last message: enable button and hide spinner
            if (continueBtn) continueBtn.disabled = false;
            
            // Show done icon in place of spinner
            if (spinner) spinner.style.display = "none";
            
            if (!doneIcon) {
                doneIcon = document.createElement('img');
                doneIcon.src = '/execution/web/icons/exec-plan-done.svg';
                doneIcon.alt = 'Execution Plan Done';
                doneIcon.style.width = '3rem';
                doneIcon.style.height = '3rem';
                doneIcon.className = 'h-12 w-12'; // Add Tailwind classes
                
                // Insert the done icon in the spinner container
                spinnerContainer.appendChild(doneIcon);
            } else {
                doneIcon.style.display = "";
            }
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
            const published = appManager.publish('plugin_execplan', 'ExecutionPlanCompleted', 
                { status: 'completed' }
            );
            
            if (published) {
                console.log("[plugin_execplan] Completion status published");
                statusContent.innerHTML = '<div class="alert alert-success">Setup complete! Redirecting...</div>';
                continueBtn.disabled = true;
            } else {
                console.warn("[plugin_execplan] Completion publish failed");
                statusContent.innerHTML = '<div class="alert alert-warning">Failed to complete. Please try again.</div>';
            }
        });
    }
    
    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_execplan');
    };
}