import { invokePayload } from "../utils";
import { UpowerProperties } from "@bindings/UpowerProperties";
import { listen } from "@tauri-apps/api/event";
import { createIconMask } from "../components/iconImage";

let battery: HTMLDivElement | null = null;
let batteryText: HTMLSpanElement | null = null;
let batteryImage: HTMLDivElement | null = null;

export function initBattery() {
    battery = document.getElementById("battery") as HTMLDivElement | null;
    if(!battery) return;

    batteryText = document.createElement("span");
    battery.appendChild(batteryText);
    
    (async () => {
        const initialValue = await invokePayload<undefined, UpowerProperties>("plugin:bar|get_upower_properties", undefined);
        battery.style.display = initialValue.is_present ? "flex" : "none";

        updateBattery(initialValue);
        batteryImage = createIconMask(initialValue.icon_name, "Adwaita", true);
        batteryImage.className = "gradient-image icon";
        battery.prepend(batteryImage);
    })();

    listen<UpowerProperties>("upower_properties_changed", (event) => {
        updateBattery(event.payload);
    });
}

// Formats a time in seconds.
function formatTime(time: number | bigint) {
    if(typeof time === "bigint") {
        // This number doesn't get huge anyway
        time = Number(time);
    }

    if(time < 0) return "unknown";

    const minutes = Number(time / 60);
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = Math.round(minutes % 60);

    if(hours > 0) {
        return `${hours}h ${remainingMinutes}m`;
    } else {
        return `${remainingMinutes}m`;
    }
}

function updateBattery(properties: UpowerProperties) {
    if(!battery || !batteryText) return;

    battery.style.display = properties.is_present ? "flex" : "none";

    batteryText.innerText = `${Math.round(properties.percentage)}%`;
    if(batteryImage) {
        batteryImage.dataset.icon = properties.icon_name;
    }

    let title = `${properties.percentage}% charged; `;

    switch(properties.state) {
        case "Empty": title += "fully empty (???)."; break;
        case "Charging": title += `${formatTime(properties.time_to_full)} to full.`; break;
        case "Discharging": title += `${formatTime(properties.time_to_empty)} to empty.`; break;
        case "FullyCharged": title += "fully charged."; break;
        case "PendingCharge": title += "pending charge."; break;
        case "PendingDischarge": title += "pending discharge."; break;
        case "Unknown": title += "unknown state."; break;
        default: {
            const _exhaustiveCheck: never = properties.state;
            console.error(`Invalid battery state: ${_exhaustiveCheck}`);
        }
    }

    title += `\n ${Math.round(properties.energy * 10) / 10} / ${Math.round(properties.energy_full * 10) / 10} Wh; `;

    switch(properties.state) {
        case "Charging": title += `charging at ${Math.round(properties.energy_rate * 10) / 10}W.`; break;
        case "Discharging": title += `discharging at ${Math.round(properties.energy_rate * 10) / 10}W.`; break;
        default: title += "not charging nor discharging.";
    }

    battery.title = title;
}