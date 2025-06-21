import { listen } from "@tauri-apps/api/event";
import { debug_log as debugLog, invokePayload } from "./utils";
import { Request } from "@bindings/NiriIpcRequest";
import { Output, Response, Window, Workspace } from "@bindings/NiriIpcResponse";
import { Event } from "@bindings/NiriIpcEvent";
import { animateTextChange } from "./textAnimation";

// TODO: This may be better to handle state for in Rust because then it can be shared between outputs.

let outputs: {
    [k: string]: Output
} = {};

let windows: Window[] = [];
let workspaces: Workspace[] = [];
let overviewOpen = false;

let windowsElement: HTMLDivElement | null = null;
let workspacesElement: HTMLDivElement | null = null;
let activeWindowTitleElement: HTMLSpanElement | null = null;

// Convert serde's { Type: { ... } } to a more TS-friendly type { type: "Type", data: { ... } }

type KeysOfUnion<T> = T extends T ? keyof T: never;
type KeyInUnion<T, K extends string> = T extends Record<K, infer U> ? U : never;
type EventType = KeysOfUnion<Event>;
type TypedEvent = {
    [K in EventType]: { type: K; data: KeyInUnion<Event, K> };
}[EventType];

function convertEvent(event: Event): TypedEvent {
    const type = Object.keys(event)[0] as EventType;
    return {
        type,
        data: event[type as keyof Event] as KeyInUnion<Event, EventType>
    };
}

function getOutputWorkspaces(output: string) {
    return workspaces.filter(w => w.output === output);
}
function getWorkspaceWindows(workspace: number) {
    return windows.filter(w => w.workspace_id === workspace);
}

export function initializeNiri() {
    listen("niri_event", (event) => {
        const e = convertEvent(event.payload as Event);

        switch(e.type) {
            case "WorkspacesChanged": {
                workspaces = e.data.workspaces;
                updateWorkspaceWidgets();
                break;
            }
            case "WorkspaceActiveWindowChanged": {
                const workspaceIndex = workspaces.findIndex(w => w.id === e.data.workspace_id);
                if(workspaceIndex !== -1) {
                    workspaces[workspaceIndex].active_window_id = e.data.active_window_id;
                } else {
                    debugLog(`Workspace with ID ${e.data.workspace_id} not found.`);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "WorkspaceActivated": {
                const workspaceToActivate = workspaces.find(w => w.id === e.data.id);
                if(workspaceToActivate) {
                    // Each output has its own active workspace
                    const outputId = workspaceToActivate.output;
                    workspaces.forEach(w => {
                        if(w.output === outputId) {
                            w.is_active = false;
                        }
                    });
                    workspaceToActivate.is_active = true;

                    // There's one focused workspace between all outputs
                    if(e.data.focused) {
                        workspaces.forEach(w => w.is_focused = false);
                        workspaceToActivate.is_focused = true;
                    }
                    updateWorkspaceWidgets();
                } else {
                    debugLog(`Workspace with ID ${e.data.workspace_id} not found.`);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "WorkspaceUrgencyChanged": {
                const workspaceIndex = workspaces.findIndex(w => w.id === e.data.id);
                if(workspaceIndex !== -1) {
                    workspaces[workspaceIndex].is_urgent = e.data.urgent;
                } else {
                    debugLog(`Urgency change for unknown workspace ID ${e.data.id}`);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "WindowsChanged": {
                windows = e.data.windows;
                updateWorkspaceWidgets();
                break;
            }
            case "WindowClosed": {
                windows = windows.filter(w => w.id != e.data.id);
                updateWorkspaceWidgets();
                break;
            }
            case "WindowFocusChanged": {
                const windowIndex = windows.findIndex(w => w.id === e.data.id);
                if(windowIndex !== -1) {
                    // Set the focused window
                    windows.forEach(w => w.is_focused = false);
                    windows[windowIndex].is_focused = true;

                    // Update the workspace's active window if needed
                    const workspaceIndex = workspaces.findIndex(w => w.id === windows[windowIndex].workspace_id);
                    if(workspaceIndex !== -1) {
                        workspaces[workspaceIndex].active_window_id = windows[windowIndex].id;
                    }
                } else {
                    // No window is focused
                    windows.forEach(w => w.is_focused = false);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "WindowOpenedOrChanged": {
                const existingWindowIndex = windows.findIndex(w => w.id === e.data.window.id);
                if(existingWindowIndex !== -1) {
                    windows[existingWindowIndex] = e.data.window;
                } else {
                    windows.push(e.data.window);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "WindowUrgencyChanged": {
                const urgencyWindowIndex = windows.findIndex(w => w.id === e.data.id);
                if(urgencyWindowIndex !== -1) {
                    windows[urgencyWindowIndex].is_urgent = e.data.urgent;
                } else {
                    debugLog(`Urgency change for unknown window ID ${e.data.id}`);
                }
                updateWorkspaceWidgets();
                break;
            }
            case "OverviewOpenedOrClosed": {
                overviewOpen = e.data.is_open;
                updateWorkspaceWidgets();
                break;
            }
            case "KeyboardLayoutSwitched":
            case "KeyboardLayoutsChanged": {
                // For now, we don't care about keyboard layouts
                break;
            }
            default: {
                debugLog(`Received unknown niri event: ${JSON.stringify(event.payload)}`);
            }
        }
    });
    
    (async () => {
        const outputsResponse = await niriRequest("Outputs");
        if(typeof outputsResponse === "object" && "Outputs" in outputsResponse) {
            outputs = outputsResponse.Outputs;
        }

        const windowsResponse = await niriRequest("Windows");
        if(typeof windowsResponse === "object" && "Windows" in windowsResponse) {
            windows = windowsResponse.Windows;
            updateWorkspaceWidgets();
        }

        const workspacesResponse = await niriRequest("Workspaces");
        if(typeof workspacesResponse === "object" && "Workspaces" in workspacesResponse) {
            workspaces = workspacesResponse.Workspaces;
            updateWorkspaceWidgets();
        }
    })();

    workspacesElement = document.getElementById("niriWorkspaces") as HTMLDivElement | null;
    windowsElement = document.getElementById("niriWindows") as HTMLDivElement | null;
    activeWindowTitleElement = document.getElementById("activeWindowTitle") as HTMLSpanElement | null;
}

function updateWorkspaceWidgets() {
    if(workspacesElement === null || windowsElement === null || activeWindowTitleElement === null) return;

    // TODO: Update when supporting multiple monitors
    const associatedOutput = Object.keys(outputs)[0];
    const outputWorkspaces = getOutputWorkspaces(associatedOutput);
    const activeWorkspace = outputWorkspaces.find(w => w.is_active);
    if(!activeWorkspace) {
        debugLog("No active workspace found on the associated output.");
        return;
    }

    const workspaceWindows = getWorkspaceWindows(activeWorkspace.id);

    const oldWindowsWidth = windowsElement.offsetWidth;

    // Niri doesn't tell us where windows are, so we unfortunately can't sort them    
    if(windowsElement.children.length > workspaceWindows.length) {
        // Remove excess windows
        while(windowsElement.children.length > workspaceWindows.length) {
            windowsElement.removeChild(windowsElement.lastChild!);
        }
    }

    for(let i = 0; i < workspaceWindows.length; i++) {
        const window = workspaceWindows[i];

        let child: HTMLLIElement;
        if(windowsElement.children.length <= i) {
            // Add new windows if there are fewer than expected
            child = document.createElement("li");
            const bullet = document.createElement("div");
            bullet.classList = "bullet";
            child.appendChild(bullet);
            windowsElement.appendChild(child);
        } else {
            child = windowsElement.children[i] as HTMLLIElement;
        }

        child.className = `niriWindow ${window.is_focused ? "focused" : ""} ${window.is_urgent ? "urgent" : ""}`;
        child.title = `${window.title || "No title"}
${windows.length} windows total; ${workspaceWindows.length} on workspace`;
    }

    const sortedWorkspaces = outputWorkspaces.sort((w1, w2) => w1.idx - w2.idx);
    if(workspacesElement.children.length > sortedWorkspaces.length) {
        // Remove excess workspaces
        while(workspacesElement.children.length > workspaceWindows.length) {
            workspacesElement.removeChild(workspacesElement.lastChild!);
        }
    }

    for(let i = 0; i < sortedWorkspaces.length; i++) {
        const workspace = sortedWorkspaces[i];

        let child: HTMLLIElement;
        if(workspacesElement.children.length <= i) {
            // Add new windows if there are fewer than expected
            child = document.createElement("li");
            const bullet = document.createElement("div");
            bullet.classList = "bullet";
            child.appendChild(bullet);
            workspacesElement.appendChild(child);
        } else {
            child = workspacesElement.children[i] as HTMLLIElement;
        }

        child.className = `niriWorkspace ${
            workspace.is_active ? "active" : ""} ${
            workspace.is_focused ? "focused" : ""} ${
            workspace.is_urgent ? "urgent" : ""}`;
        child.title = `${workspace.name || `Workspace ${workspace.idx}`} | ${getWorkspaceWindows(workspace.id).length} windows
${workspaces.length} workspaces total; ${outputWorkspaces.length} on output`;
    }

    // Translate the windowsElement's width from its previous value to the new required one
    const newWindowsWidth = windowsElement.offsetWidth;
    if(oldWindowsWidth != newWindowsWidth) {
        windowsElement.animate([
            { width: `${oldWindowsWidth}px` },
            { width: `${newWindowsWidth}px` }
        ], {
            duration: 200,
            easing: "ease-in-out"
        });
    }

    // Update the active window title
    const activeWindow = workspaceWindows.find(w => w.is_focused);
    if(activeWindow && activeWindow.title) {
        animateTextChange(activeWindowTitleElement, activeWindow.title);
        activeWindowTitleElement.title = `${activeWindow.title} | ${activeWindow.id}`;
    } else if(overviewOpen) {
        animateTextChange(activeWindowTitleElement, "Overview");
    } else {
        animateTextChange(activeWindowTitleElement, "-");
    }
}

async function niriRequest(request: Request): Promise<Response> {
    return invokePayload("plugin:bar|niri_request", request);
}