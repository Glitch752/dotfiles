let clock: HTMLSpanElement | null = null;

export function initWidgets() {
    clock = document.getElementById("clock");
}

export function updateWidgets() {
    if(!clock) return;
    clock.innerText = new Date().toLocaleString([], {
        day: "2-digit",
        month: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: true
    }).replace(",", "");

    clock.title = new Date().toLocaleString([], {
        day: "2-digit",
        weekday: "long",
        month: "long",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        hour12: true,
        timeZoneName: "short"
    }).replace(",", "");

    // TODO: Calendar popup
}