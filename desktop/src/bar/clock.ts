let clock: HTMLSpanElement | null = null;

export function initClock() {
    clock = document.getElementById("clock");

    updateClock();
    setInterval(updateClock, 1000);
}

function updateClock() {
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