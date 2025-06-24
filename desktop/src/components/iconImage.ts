import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { invokePayload } from "../utils";

const resolvedIconCache = new Map<string, string>();

export function createIconImage(
    icon: string,
    theme: string = "hicolor",
    canChange: boolean = false,
): HTMLImageElement {
    const element = document.createElement("img");
    element.dataset.icon = icon;

    if(resolvedIconCache.has(icon)) {
        element.src = resolvedIconCache.get(icon)!;
    } else {
        // Does this need to be a weakref? Probably not... but weak pointers are a cool feature.
        const ref = new WeakRef(element);
        invoke<string>("plugin:launcher|resolve_icon", {
            icon,
            theme
        }).then((source) => {
            const deref = ref.deref();
            const src = convertFileSrc(source);
            resolvedIconCache.set(icon, src);
            if(deref) deref.src = src;
        });
    }

    if(canChange) {
        // Mutation observers only hold weak references
        // Observe the data-icon attribute and reload the icon if it changes
        const observer = new MutationObserver((mutations) => {
            for(const mutation of mutations) {
                if(mutation.type === "attributes" && mutation.attributeName === "data-icon") {
                    const target = mutation.target as HTMLImageElement;
                    const newIcon = target.dataset.icon ?? "";
                    if(newIcon && resolvedIconCache.has(newIcon)) {
                        target.src = resolvedIconCache.get(newIcon)!;
                    } else {
                        // If the icon is not in the cache, we can just reload it
                        invokePayload<string, string>("plugin:launcher|resolve_icon", newIcon).then((source) => {
                            const src = convertFileSrc(source);
                            resolvedIconCache.set(newIcon, src);
                            target.src = src;
                        });
                    }
                }
            }
        });
        observer.observe(element, {
            attributes: true,
            attributeFilter: ["data-icon"]
        });
    }

    return element;
}

/**
 * This one is less obvious.
 * If masked, we don't draw the icon as an actual img--it's a div with a mask-image: url(...) property set.
 * This allows us to use a background gradient.  
 * The returned element doesn't have any styles by default,
 * so it's up to the caller to set a width, height, and other required background properties.
 */
export function createIconMask(
    icon: string,
    theme: string = "hicolor",
    canChange: boolean = false
): HTMLDivElement {
    const element = document.createElement("div");
    element.dataset.icon = icon;

    if(resolvedIconCache.has(icon)) {
        element.style.maskImage = `url(${resolvedIconCache.get(icon)!})`;
    } else {
        // Does this need to be a weakref? Probably not... but weak pointers are a cool feature.
        const ref = new WeakRef(element);
        invoke<string>("plugin:launcher|resolve_icon", {
            icon,
            theme
        }).then((source) => {
            const deref = ref.deref();
            const src = convertFileSrc(source);
            resolvedIconCache.set(icon, src);
            if(deref) deref.style.maskImage = `url(${src}`;
        });
    }

    if(canChange) {
        // Mutation observers only hold weak references
        // Observe the data-icon attribute and reload the icon if it changes
        const observer = new MutationObserver((mutations) => {
            for(const mutation of mutations) {
                if(mutation.type === "attributes" && mutation.attributeName === "data-icon") {
                    const target = mutation.target as HTMLDivElement;
                    const newIcon = target.dataset.icon ?? "";
                    if(newIcon && resolvedIconCache.has(newIcon)) {
                        target.style.maskImage = `url(${resolvedIconCache.get(newIcon)!})`;
                    } else {
                        // If the icon is not in the cache, we can just reload it
                        invokePayload<string, string>("plugin:launcher|resolve_icon", newIcon).then((source) => {
                            const src = convertFileSrc(source);
                            resolvedIconCache.set(newIcon, src);
                            target.style.maskImage = `url(${src})`;
                        });
                    }
                }
            }
        });
        observer.observe(element, {
            attributes: true,
            attributeFilter: ["data-icon"]
        });
    }

    return element;
}