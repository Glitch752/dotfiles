import { createIconMask } from "../components/iconImage";
import { openPopup, PopupType } from "../popups/popups";

export function initPowerOptions() {
    const powerOptions = document.getElementById("powerOptions");
    if(!powerOptions) return;

    const icon = createIconMask("system-shutdown-symbolic", "Adwaita");
    icon.className = "icon gradient-image";
    powerOptions.append(icon);

    powerOptions.onmousedown = () => {
        const content = document.createElement("div");
        content.style.width = "200px";
        content.style.height = "50px";
        content.textContent = "TODO: Power options";
        openPopup(PopupType.LeftPanel, "powerOptions", content, powerOptions);
    };
}