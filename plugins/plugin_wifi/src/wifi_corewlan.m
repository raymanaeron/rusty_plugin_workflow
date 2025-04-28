#import <CoreWLAN/CoreWLAN.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *ssid;
    const char *bssid;
    int signalStrength;
    int channel;
    const char *security;
} WiFiNetwork;

WiFiNetwork *scan_wifi_networks(size_t *out_count) {
    CWInterface *wifiInterface = [[CWWiFiClient sharedWiFiClient] interface];
    if (!wifiInterface) {
        NSLog(@"[plugin_wifi] No Wi-Fi interface found.");
        *out_count = 0;
        return NULL;
    }

    NSLog(@"[plugin_wifi] Using Wi-Fi interface: %@", wifiInterface.interfaceName);

    NSError *error = nil;
    NSSet<CWNetwork *> *networks = [wifiInterface scanForNetworksWithSSID:nil error:&error];

    if (error) {
        NSLog(@"[plugin_wifi] Error scanning for networks: %@", error.localizedDescription);
        *out_count = 0;
        return NULL;
    }

    if (!networks || networks.count == 0) {
        NSLog(@"[plugin_wifi] No networks found.");
        *out_count = 0;
        return NULL;
    }

    WiFiNetwork *results = malloc(sizeof(WiFiNetwork) * networks.count);
    size_t index = 0;

    for (CWNetwork *network in networks) {
        const char *logPath = "/tmp/plugin_wifi_debug.log";

        NSLog(@"[plugin_wifi] Attempting to write to log file: %s", logPath);

        FILE *logFile = fopen(logPath, "a");
        if (!logFile) {
            perror("[plugin_wifi] Failed to open log file in /tmp");
        } else {
            fprintf(logFile, "[plugin_wifi] CWNetwork object: %s\n", [[network description] UTF8String]);
            fclose(logFile);
            NSLog(@"[plugin_wifi] Successfully wrote to log file: %s", logPath);
        }

        const char *ssid = network.ssid ? network.ssid.UTF8String : "Hidden Network";
        const char *bssid = network.bssid ? network.bssid.UTF8String : "Unknown BSSID";

        results[index].ssid = strdup(ssid);
        results[index].bssid = strdup(bssid);
        results[index].signalStrength = network.rssiValue;
        results[index].channel = network.wlanChannel.channelNumber;

        // Use the `supportsSecurity:` method to determine the security type
        if ([network supportsSecurity:kCWSecurityWPA2Personal]) {
            results[index].security = strdup("WPA2 Personal");
        } else if ([network supportsSecurity:kCWSecurityWPA3Personal]) {
            results[index].security = strdup("WPA3 Personal");
        } else if ([network supportsSecurity:kCWSecurityNone]) {
            results[index].security = strdup("None");
        } else {
            results[index].security = strdup("Unknown");
        }
        index++;
    }

    *out_count = networks.count;
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
