export async function activate(container, appManager) {
    // Check if container has been properly initialized
    if (!container) {
        console.error('Container is null or undefined');
        return;
    }
    
    // Register with app manager
    appManager.registerPlugin('plugin_settings');
    console.log('Plugin activated: plugin_settings');

    // Get UI elements
    const submitBtn = container.querySelector('#submitBtn');
    const clearBtn = container.querySelector('#clearBtn');
    const resultBox = container.querySelector('#resultBox');

    // Check for critical elements
    if (!submitBtn || !clearBtn || !resultBox) {
        console.error('Critical UI elements not found in container:', 
            { submitBtn: !!submitBtn, clearBtn: !!clearBtn, resultBox: !!resultBox });
        return;
    }
    
    // Define the device settings data structure
    const deviceSettings = {
        general: {
            deviceName: "My Echo",
            language: "en-US",
            region: "us",
            timeZone: "GMT-5",
            autoUpdate: true,
            amazonEmail: "",
            shareMetrics: true
        },
        echo: {
            wakeWord: "Alexa",
            micEnabled: true,
            dropInCalling: false,
            displaySettings: "brightness"
        },
        automation: {
            frustrationFreeAutomation: true
        }
    };

    // Function to validate email
    function isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    async function postData(payload) {
        try {
            const response = await fetch('/api/settings/devicesettings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            const data = await response.json();
            if (response.ok) {
                console.log('Data posted successfully:', data);
                return data;
            } else {
                console.error('Failed to post data:', data);
                throw new Error(data.message || 'Failed to post data');
            }
        } catch (error) {
            console.error('Error posting data:', error);
            throw error;
        }
    }

    // Initialize form with default values
    function initializeForm() {
        // General section
        container.querySelector("#deviceName").value = deviceSettings.general.deviceName;
        container.querySelector("#language").value = deviceSettings.general.language;
        container.querySelector("#region").value = deviceSettings.general.region;
        container.querySelector("#timeZone").value = deviceSettings.general.timeZone;
        container.querySelector("#autoUpdate").checked = deviceSettings.general.autoUpdate;
        container.querySelector("#amazonEmail").value = deviceSettings.general.amazonEmail;
        container.querySelector("#shareMetrics").checked = deviceSettings.general.shareMetrics;
        
        // Echo section
        container.querySelector("#wakeWord").value = deviceSettings.echo.wakeWord;
        container.querySelector("#micEnabled").checked = deviceSettings.echo.micEnabled;
        container.querySelector("#dropInCalling").checked = deviceSettings.echo.dropInCalling;
        container.querySelector("#displaySettings").value = deviceSettings.echo.displaySettings;
        
        // Automation section
        container.querySelector("#frustrationFreeAutomation").checked = deviceSettings.automation.frustrationFreeAutomation;
    }

    // Update data structure from form
    function updateSettingsFromForm() {
        // General section
        deviceSettings.general.deviceName = container.querySelector("#deviceName").value;
        deviceSettings.general.language = container.querySelector("#language").value;
        deviceSettings.general.region = container.querySelector("#region").value;
        deviceSettings.general.timeZone = container.querySelector("#timeZone").value;
        deviceSettings.general.autoUpdate = container.querySelector("#autoUpdate").checked;
        deviceSettings.general.amazonEmail = container.querySelector("#amazonEmail").value;
        deviceSettings.general.shareMetrics = container.querySelector("#shareMetrics").checked;
        
        // Echo section
        deviceSettings.echo.wakeWord = container.querySelector("#wakeWord").value;
        deviceSettings.echo.micEnabled = container.querySelector("#micEnabled").checked;
        deviceSettings.echo.dropInCalling = container.querySelector("#dropInCalling").checked;
        deviceSettings.echo.displaySettings = container.querySelector("#displaySettings").value;
        
        // Automation section
        deviceSettings.automation.frustrationFreeAutomation = container.querySelector("#frustrationFreeAutomation").checked;
        
        return deviceSettings;
    }

    // Setup email validation
    const emailField = container.querySelector("#amazonEmail");
    if (emailField) {
        emailField.addEventListener("input", function() {
            if (this.value && !isValidEmail(this.value)) {
                emailField.classList.add("is-invalid");
            } else {
                emailField.classList.remove("is-invalid");
            }
        });
    }

    // Initialize form
    initializeForm();

    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            // Reset the form fields to initial values
            initializeForm();
            const emailField = container.querySelector("#amazonEmail");
            if (emailField) {
                emailField.classList.remove("is-invalid");
            }
        });
    }

    if (submitBtn) {
        submitBtn.addEventListener('click', async () => {
            // Validate email before submission
            const emailField = container.querySelector("#amazonEmail");
            if (emailField && emailField.value && !isValidEmail(emailField.value)) {
                emailField.classList.add("is-invalid");
                return;
            }
            
            // Update settings object from form values
            const updatedSettings = updateSettingsFromForm();
            
            // Log the settings to console
            console.log("Device Settings:", updatedSettings);

            // Example: POST data to API
            try {
                await postData(updatedSettings);
                console.log('Settings submitted successfully');
                resultBox.innerText = 'Settings submitted successfully!';
                resultBox.classList.remove('hidden');
            } catch (error) {
                console.error(`Save settings failed: ${error.message}`);
                resultBox.innerText = `Error: ${error.message}`;
                resultBox.classList.remove('hidden');
            }

            const published = appManager.publish('plugin_settings', 'SettingsCompleted',
                { status: 'completed' }
            );
        });
    }

    // Return cleanup function at module level
    return () => {
        appManager.unregisterPlugin('plugin_settings');
    }
}
