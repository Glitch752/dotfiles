import { invokePayload } from "../utils";
import { UpowerProperties } from "@bindings/UpowerProperties";
import { listen } from "@tauri-apps/api/event";
import { createIconMask } from "../components/iconImage";

let batteryText: HTMLSpanElement | null = null;
let batteryImage: HTMLDivElement | null = null;

export function initBattery() {
    const battery = document.getElementById("battery") as HTMLDivElement | null;
    if(!battery) return;

    batteryText = document.createElement("span");
    battery.appendChild(batteryText);
    
    (async () => {
        const initialValue = await invokePayload<undefined, UpowerProperties>("plugin:bar|get_upower_properties", undefined);
        updateBattery(initialValue);
        batteryImage = createIconMask(initialValue.icon_name, "Adwaita");
        batteryImage.className = "gradient-image icon";
        battery.prepend(batteryImage);
    })();

    listen<UpowerProperties>("upower_properties_changed", (event) => {
        updateBattery(event.payload);
    });
}

function updateBattery(properties: UpowerProperties) {
    if(!batteryText) return;

    batteryText.innerText = `${Math.round(properties.percentage)}%`;
    if(batteryImage) {
        batteryImage.dataset.icon = properties.icon_name;
    }
}