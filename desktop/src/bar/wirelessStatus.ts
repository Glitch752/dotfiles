import { listen } from "@tauri-apps/api/event";
import { createIconMask } from "../components/iconImage";
import { invokePayload } from "../utils";
import { NetworkManagerState, NetworkStatus } from "@bindings/NetworkManagerState";

let networkStatus: HTMLDivElement | null = null;
let networkStatusIcon: HTMLDivElement | null = null;

export function initNetworkStatus() {
    networkStatus = document.getElementById("networkStatus") as HTMLDivElement | null;
    if(!networkStatus) return;

    (async () => {
        const initialValue = await invokePayload<undefined, NetworkManagerState>("plugin:bar|get_networkmanager_state", undefined);

        updateStatus(initialValue);
        networkStatusIcon = createIconMask(getIcon(initialValue.status), "Adwaita", true);
        networkStatusIcon.className = "gradient-image icon";
        networkStatus.append(networkStatusIcon);
    })();

    listen<NetworkManagerState>("networkmanager_state_changed", (event) => {
        updateStatus(event.payload);
    });
}

function getIcon(status: NetworkStatus) {
    switch(status) {
        case "WiredConnected":
            return "network-wired-symbolic";
        case "WifiConnected":
            return "network-wireless-symbolic";
        case "CellularConnected":
            return "network-cellular-symbolic";
        case "VpnConnected":
            return "network-vpn-symbolic";
        case "WifiDisconnected":
            return "network-wireless-acquiring-symbolic";
        case "Offline":
            return "network-wireless-disabled-symbolic";
        case "Unknown":
            return "dialog-question-symbolic";
    }
}

function updateStatus(state: NetworkManagerState) {
    if(!networkStatus) return;
    switch(state.status) {
        case "Offline":
            networkStatus.title = "Offline";
            break;
        case "CellularConnected":
            networkStatus.title = "Cellular connected";
            break;
        case "Unknown":
            networkStatus.title = "Network status unknown";
            break;
        case "VpnConnected":
            networkStatus.title = "VPN connected";
            break;
        case "WifiConnected":
            networkStatus.title = "Wi-Fi connected";
            break;
        case "WifiDisconnected":
            networkStatus.title = "Wi-Fi disconnected";
            break;
        case "WiredConnected":
            networkStatus.title = "Wired connected";
            break;
    }

    if(networkStatusIcon) {
        networkStatusIcon.dataset.icon = getIcon(state.status);
    }
}