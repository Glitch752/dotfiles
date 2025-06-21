import addLauncherPopup from "./launcher/lancherSearch";
import { closePopup, openPopup, PopupType } from "./popups";

let archLogo: HTMLElement | null = null;

export function initLauncher() {
    archLogo = document.getElementById("archlogo");

    if(archLogo) {
        archLogo.addEventListener("click", () => {
            openLauncher();
        });
    }
}

export function openLauncher() {
    if(!archLogo) return;
    
    const content = document.createElement("div");
    content.className = "launcher";

    addLauncherPopup(content);

    openPopup(PopupType.Launcher, "launcher", content, archLogo);
}

export function closeLauncher() {
    closePopup(PopupType.Launcher, "launcher");
}