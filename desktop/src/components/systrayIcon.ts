import { createIconMask } from "./iconImage";
import type { SystrayIcon } from "@bindings/SystemTray";

export function createSystrayIconElement(icon: SystrayIcon): HTMLElement {
    let gradientIcon;
    if("FreedesktopIcon" in icon) {
        const { theme, name } = icon.FreedesktopIcon;
        gradientIcon = createIconMask(name, theme, false);
    } else if("Pixmaps" in icon) {
        // Just use the first pixmap as the icon image for now
        // TODO: better selection logic
        const pixmap = icon.Pixmaps.icons[0];
        
        gradientIcon = document.createElement("div");

        const canvas = document.createElement("canvas");
        canvas.width = pixmap.width;
        canvas.height = pixmap.height;
        const ctx = canvas.getContext("2d");
        if(ctx) {
            // Convert ARGB32 (network byte order) to RGBA for canvas
            const pixelData = new Uint8ClampedArray(pixmap.width * pixmap.height * 4);
            for(let i = 0; i < pixelData.length; i += 4) {
                // ARGB32 (network order): [A, R, G, B]
                pixelData[i]     = pixmap.pixels[i + 1]; // R
                pixelData[i + 1] = pixmap.pixels[i + 2]; // G
                pixelData[i + 2] = pixmap.pixels[i + 3]; // B
                pixelData[i + 3] = pixmap.pixels[i];     // A
            }
            const imageData = ctx.createImageData(pixmap.width, pixmap.height);
            imageData.data.set(pixelData);
            ctx.putImageData(imageData, 0, 0);
            gradientIcon.style.maskImage = `url(${canvas.toDataURL()})`;
        }
    } else {
        throw new Error("Unknown SystrayIcon variant");
    }

    gradientIcon.className = "icon gradient-image";
    
    const container = document.createElement("div");

    const normalIcon = document.createElement("div");
    normalIcon.style.backgroundImage = gradientIcon.style.maskImage;

    // Observe maskImage changes. This could probably be made simpler... but meh.
    let observer = new MutationObserver((mutations) => {
        for(const mutation of mutations) {
            if(mutation.type === "attributes" && mutation.attributeName === "style") {
                normalIcon.style.backgroundImage = gradientIcon.style.maskImage;
            }
        }
    });
    observer.observe(gradientIcon, { attributes: true, attributeFilter: ["style"] });

    normalIcon.className = "icon normal-image";

    container.append(normalIcon, gradientIcon);
    container.className = "systray-icon";

    return container;
}