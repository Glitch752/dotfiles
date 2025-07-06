import { invokePayload } from "../utils";
import { SystemTrayItems } from "@bindings/SystemTray";
import { listen } from "@tauri-apps/api/event";

let systrayItems: HTMLDivElement | null = null;

export function initSystray() {
    systrayItems = document.getElementById("systrayItems") as HTMLDivElement | null;
    if(!systrayItems) return;

    (async () => {
        const initialItems = await invokePayload<undefined, SystemTrayItems>("plugin:bar|get_systray_items", undefined);
        updateItems(initialItems);
    })();

    listen<SystemTrayItems>("update_tray_items", (event) => {
        updateItems(event.payload);
    });
}

function updateItems(items: SystemTrayItems) {
    if(!systrayItems) return;

    console.log(items);

    systrayItems.innerHTML = `${Object.values(items).map(item => {
        // return `<div title="${item?.tooltip?.title}">${item?.title}</div>`;
        return "";
    }).join("")}`;
}