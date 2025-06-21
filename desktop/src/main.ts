import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { ExclusiveRegions } from "@bindings/ExclusiveRegions";
import { InputRect } from "@bindings/InputRect";
import { init } from "./rendering";
import { initWidgets, updateWidgets } from "./widgets";
import { invokePayload } from "./utils";
import { initializeNiri } from "./niri";

// Get bar thicknesses from :root in CSS
const root = document.querySelector(":root") as HTMLElement;
export const barThickness = parseInt(getComputedStyle(root).getPropertyValue("--bar-thickness").trim());
export const nonBorderBarThickness = parseInt(getComputedStyle(root).getPropertyValue("--non-bar-border-thickness").trim());

window.addEventListener("DOMContentLoaded", () => {
    console.log("Loaded");

    updateInputShape([]);
    init();
    initWidgets();
    initializeNiri();

    invokePayload<ExclusiveRegions>("create_exclusive_regions", {
        top: barThickness,
        bottom: nonBorderBarThickness,
        left: barThickness,
        right: nonBorderBarThickness
    });

    updateWidgets();
    setInterval(updateWidgets, 1000);
});

function updateInputShape(extraRectangles: InputRect[]) {
    invokePayload<InputRect[]>("set_input_shape", [
        {
            x: 0, y: 0,
            width: window.innerWidth, height: barThickness
        },
        {
            x: 0, y: window.innerHeight - nonBorderBarThickness,
            width: window.innerWidth, height: nonBorderBarThickness
        },
        {
            x: 0, y: barThickness,
            width: barThickness, height: window.innerHeight - barThickness - nonBorderBarThickness
        },
        {
            x: window.innerWidth - nonBorderBarThickness, y: barThickness,
            width: nonBorderBarThickness, height: window.innerHeight - barThickness - nonBorderBarThickness
        },
        ...extraRectangles
    ]);
}

window.addEventListener("resize", () => {
    updateInputShape([]);
});

listen<string>("ipc_call", (event) => {
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
            response = "Reset to full screen for 5 seconds.";
            setTimeout(() => {
                updateInputShape([]);
            }, 5000);
    }
    // Outside the switch statement to guarentee this is run
    emit("ipc_response", response);
});