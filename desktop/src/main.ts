import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { ExclusiveRegions } from "@bindings/ExclusiveRegions";
import { Notification } from "@bindings/Notification";
import { init } from "./rendering";
import { initClock } from "./bar/clock";
import { invokePayload, debugLog } from "./utils";
import { initializeNiri as initNiri } from "./bar/niri";
import { updateInputShape } from "./popups/popups";
import { initLauncher, openLauncher } from "./popups/launcher";
import { exit, relaunch } from "@tauri-apps/plugin-process";
import { initNotifications } from "./popups/notifications";
import { initBattery } from "./bar/battery";
import { initPowerOptions } from "./bar/powerOptions";
import { initNetworkStatus } from "./bar/wirelessStatus";
import { initSystray } from "./bar/systray";

// Get bar thicknesses from :root in CSS
const root = document.querySelector(":root") as HTMLElement;
export const barThickness = parseInt(getComputedStyle(root).getPropertyValue("--bar-thickness").trim());
export const nonBorderBarThickness = parseInt(getComputedStyle(root).getPropertyValue("--non-bar-border-thickness").trim());

export let overrideInputShape = false;

// Hook console.* to use our debugLog
function hook(method: keyof Console) {
    const original = console[method];
    console[method] = (...args: any[]) => {
        debugLog(`[${method}]`, ...args);
        original.apply(console, args);
    };
}
hook("log");
hook("warn");
hook("error");
hook("info");

window.addEventListener("DOMContentLoaded", () => {
    updateInputShape([]);
    init();
    initClock();
    initSystray();
    initNetworkStatus();
    initPowerOptions();
    initBattery();
    initLauncher();
    initNotifications();
    initNiri();

    invokePayload<ExclusiveRegions>("create_exclusive_regions", {
        top: barThickness,
        bottom: nonBorderBarThickness,
        left: barThickness,
        right: nonBorderBarThickness
    });
});

window.addEventListener("resize", () => {
    updateInputShape(null);
});

listen<string>("ipc_call", async (event) => {
    let response = `Unknown event '${event.payload}'.`;
    switch(event.payload) {
        case "inspect":
            invoke("inspect");
            response = "ok";
            break;
        case "open_devtools":
            invokePayload<boolean>("devtools", true);
            response = "ok";
            break;
        case "close_devtools":
            invokePayload<boolean>("devtools", false);
            response = "ok";
            break;
        case "temporary_full_input":
            updateInputShape([{
                x: 0,
                y: 0,
                width: window.innerWidth,
                height: window.innerHeight
            }]);
            overrideInputShape = true;
            response = "Reset to full screen for 5 seconds.";
            setTimeout(() => {
                overrideInputShape = false;
                updateInputShape(null);
            }, 10000);
            break;
        case "launcher":
            openLauncher();
            response = "ok";
            break;
        case "relaunch":
            setTimeout(() => {
                relaunch();
            }, 10);
            response = "Relaunching.";
            break;
        case "reload":
            await invoke("plugin:launcher|reload_desktop_files");
            // Show a fake notification since this can be invoked with a keybind
            emit<Notification>("notification_added", {
                title: "Reloaded successfully!",
                body: "",
                actions: [],
                id: Date.now(),
                application_icon: null,
                application_name: "",
                urgency: "Low"
            })
            response = "Reloaded.";
            break;
        case "exit":
            setTimeout(() => {
                exit();
            }, 10);
            response = "Exiting.";
            break;
    }
    // Outside the switch statement to guarentee this is run
    emit("ipc_response", response);
});