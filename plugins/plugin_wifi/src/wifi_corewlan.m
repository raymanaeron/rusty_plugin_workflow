#import <CoreWLAN/CoreWLAN.h>
#import <os/log.h>
#import <CoreLocation/CoreLocation.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *ssid;
    const char *bssid;
    int signalStrength;
    int channel;
    const char *security;
} WiFiNetwork;

static os_log_t plugin_wifi_log;

__attribute__((constructor))
static void initialize_logging() {
    plugin_wifi_log = os_log_create("com.rusty_plugin_workflow.plugin_wifi", "WiFiCoreWLAN");
}

WiFiNetwork *scan_wifi_networks(size_t *out_count) {
    // Use stderr for console output as it's more likely to show up
    fprintf(stderr, "[plugin_wifi] Starting WiFi scan...\n");
    
    // Get the shared WiFi client
    CWWiFiClient *wifiClient = [CWWiFiClient sharedWiFiClient];
    if (!wifiClient) {
        fprintf(stderr, "[plugin_wifi] Failed to get WiFi client\n");
        *out_count = 0;
        return NULL;
    }
    
    CWInterface *wifiInterface = [wifiClient interface];
    if (!wifiInterface) {
        fprintf(stderr, "[plugin_wifi] No Wi-Fi interface found.\n");
        *out_count = 0;
        return NULL;
    }

    fprintf(stderr, "[plugin_wifi] Using Wi-Fi interface: %s\n", [wifiInterface.interfaceName UTF8String]);
    
    // Check if WiFi is powered on
    if (![wifiInterface powerOn]) {
        fprintf(stderr, "[plugin_wifi] WiFi is powered off\n");
        *out_count = 0;
        return NULL;
    }

    // Try to get current network information using CoreWLAN API
    CWNetwork *currentNetwork = wifiInterface.cachedScanResults.anyObject;
    if (currentNetwork) {
        fprintf(stderr, "[plugin_wifi] Current network: %s\n", [[currentNetwork description] UTF8String]);
    } else {
        fprintf(stderr, "[plugin_wifi] No cached network information available\n");
    }

    NSError *error = nil;
    
    // Use direct synchronous scanning for simplicity
    fprintf(stderr, "[plugin_wifi] Starting synchronous scan...\n");
    NSSet<CWNetwork *> *networks = [wifiInterface scanForNetworksWithSSID:nil error:&error];
    
    if (error) {
        fprintf(stderr, "[plugin_wifi] Scan error: %s\n", [error.localizedDescription UTF8String]);
        *out_count = 0;
        return NULL;
    }
    
    if (!networks || networks.count == 0) {
        fprintf(stderr, "[plugin_wifi] No networks found or access denied\n");
        *out_count = 0;
        return NULL;
    }

    fprintf(stderr, "[plugin_wifi] Scan completed, found %lu networks\n", (unsigned long)networks.count);

    // Try to access current network details
    NSString *currentSSID = wifiInterface.ssid;
    if (currentSSID) {
        fprintf(stderr, "[plugin_wifi] Currently connected to: %s\n", [currentSSID UTF8String]);
    }
    
    WiFiNetwork *results = malloc(sizeof(WiFiNetwork) * networks.count);
    size_t index = 0;

    for (CWNetwork *network in networks) {
        // Dump full network information to stderr
        fprintf(stderr, "[plugin_wifi] Network details: %s\n", [[network description] UTF8String]);
        
        // Create a descriptive name based on signal strength, channel and security
        // This makes networks more identifiable when SSIDs are restricted
        NSString *networkType;
        if ([network supportsSecurity:kCWSecurityNone]) {
            networkType = @"Open";
        } else if ([network supportsSecurity:kCWSecurityWPA2Enterprise] || 
                  [network supportsSecurity:kCWSecurityWPAEnterprise] ||
                  [network supportsSecurity:kCWSecurityWPA3Enterprise]) {
            networkType = @"Enterprise";
        } else {
            networkType = @"Secure";
        }
        
        // Create a name like "Strong Secure 5G (Ch 149)"
        NSString *bandType = (network.wlanChannel.channelNumber > 14) ? @"5G" : @"2.4G";
        NSString *signalDesc;
        int rssi = (int)network.rssiValue;
        
        if (rssi > -50) {
            signalDesc = @"Strong";
        } else if (rssi > -65) {
            signalDesc = @"Good";
        } else if (rssi > -75) {
            signalDesc = @"Fair";
        } else {
            signalDesc = @"Weak";
        }
        
        NSString *descriptiveName = [NSString stringWithFormat:@"%@ %@ %@ (Ch %d)", 
                                    signalDesc, networkType, bandType, 
                                    (int)network.wlanChannel.channelNumber];
        
        // Try different ways to access SSID
        NSString *ssidString = nil;
        
        // If this is the currently connected network and we know its name, use it
        if (currentSSID && [network isEqual:wifiInterface.lastAssociatedNetwork]) {
            ssidString = currentSSID;
        }
        
        // Try direct SSID access if we don't have a name yet
        if (!ssidString) {
            ssidString = network.ssid;
        }
        
        // Use our descriptive name if SSID is still not available
        const char *ssid = ssidString ? [ssidString UTF8String] : [descriptiveName UTF8String];
        
        // Generate a unique identifier for the BSSID based on channel and signal
        NSString *pseudoBssid = [NSString stringWithFormat:@"%d-%d-%zu", 
                               (int)network.wlanChannel.channelNumber, rssi, index];
        const char *bssid = network.bssid ? [network.bssid UTF8String] : [pseudoBssid UTF8String];
        
        fprintf(stderr, "[plugin_wifi] SSID: %s, BSSID: %s, RSSI: %d, Channel: %d\n", 
               ssid, bssid, rssi, (int)network.wlanChannel.channelNumber);
        
        results[index].ssid = strdup(ssid);
        results[index].bssid = strdup(bssid);
        results[index].signalStrength = rssi;
        results[index].channel = (int)network.wlanChannel.channelNumber;

        // Determine security type with proper constants
        NSString *securityType = @"Unknown";
        
        if ([network supportsSecurity:kCWSecurityNone]) {
            securityType = @"None";
        } else if ([network supportsSecurity:kCWSecurityWEP]) {
            securityType = @"WEP";
        } else if ([network supportsSecurity:kCWSecurityWPAPersonal]) {
            securityType = @"WPA Personal";
        } else if ([network supportsSecurity:kCWSecurityWPA2Personal]) {
            securityType = @"WPA2 Personal";
        } else if ([network supportsSecurity:kCWSecurityWPA3Personal]) {
            securityType = @"WPA3 Personal";
        } else if ([network supportsSecurity:kCWSecurityWPA3Enterprise] || 
                  [network supportsSecurity:kCWSecurityWPA2Enterprise] ||
                  [network supportsSecurity:kCWSecurityWPAEnterprise]) {
            securityType = @"Enterprise";
        }
        
        results[index].security = strdup([securityType UTF8String]);
        index++;
    }

    *out_count = index;
    return results;
}

void free_wifi_networks(WiFiNetwork *networks, size_t count) {
    for (size_t i = 0; i < count; i++) {
        free((void *)networks[i].ssid);
        free((void *)networks[i].bssid);
        free((void *)networks[i].security);
    }
    free(networks);
}