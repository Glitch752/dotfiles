import { InputRect } from "@bindings/InputRect";
import { Rectangle } from "../geom";
import { debugLog, fallingEdgeDebouncer, invokePayload } from "../utils";
import {
    barThickness,
    nonBorderBarThickness,
    overrideInputShape,
} from "../main";

let popupsContainer: HTMLDivElement | null = null;
let lastElapsed = 0;
let polling = false;

const continueAnimating = fallingEdgeDebouncer(500);

export enum PopupType {
    Launcher,
    LeftPanel,
    Notifications,
}

enum PopupOrigin {
    Top,
    Bottom,
    Left,
    Right,
}

const POPUP_DATA: {
    [key in PopupType]: {
        origin: PopupOrigin;
        requiresExclusiveKeyboard: boolean;
    };
} = {
    [PopupType.Launcher]: { origin: PopupOrigin.Top, requiresExclusiveKeyboard: true },
    [PopupType.LeftPanel]: { origin: PopupOrigin.Left, requiresExclusiveKeyboard: false },
    [PopupType.Notifications]: { origin: PopupOrigin.Top, requiresExclusiveKeyboard: false },
};

class OpenPopup {
    public open: boolean = true;
    private resizeObserver: ResizeObserver;

    constructor(
        private type: PopupType,
        public source: string,
        public element: HTMLDivElement,
        public rect: Rectangle | null
    ) {
        this.resizeObserver = new ResizeObserver(() => {
            if(this.open) {
                pollForChanges();
            }
        });
        this.resizeObserver.observe(this.element);
    }
    
    get origin() {
        return POPUP_DATA[this.type].origin;
    }
    get requiresExclusiveKeyboard() {
        return POPUP_DATA[this.type].requiresExclusiveKeyboard;
    }
    
    remove() {
        if(!popupsContainer) return;
        if(!this.open) return;

        this.open = false;

        const popupsRect = popupsContainer.getBoundingClientRect();
        const popupRect = this.element.getBoundingClientRect();
        
        switch(this.origin) {
            case PopupOrigin.Top:
                this.element.style.top = `${popupRect.top - popupRect.height - popupsRect.top - 2}px`;
                break;
            case PopupOrigin.Bottom:
                this.element.style.bottom = `${popupRect.bottom - popupRect.height - popupsRect.bottom - 2}px`;
                break;
            case PopupOrigin.Left:
                this.element.style.left = `${popupRect.left - popupRect.width - popupsRect.left - 2}px`;
                break;
            case PopupOrigin.Right:
                this.element.style.right = `${popupRect.right - popupRect.width - popupsRect.right - 2}px`;
                break;
        }

        this.element.style.opacity = "0";

        // Wait for the element's animations to finish
        setTimeout(() => {
            if(this.open) return;
            if(!popupsContainer) return;

            try {
                popupsContainer.removeChild(this.element);
            } catch(e) {
                debugLog("Error removing popup element:", e);
            }
            popups.delete(this.type);
        }, 500);

        updateKeyboardExclusivity();
        pollForChanges();
    }
}

const popups: Map<PopupType, OpenPopup> = new Map();
let popupRectangles: Rectangle[] = [];

type DrawCallback = (elapsed: number, rects: Rectangle[]) => void;

let drawCallback: DrawCallback | null = null;

export function closePopup(type: PopupType, source: string) {
    if(!popupsContainer) return;

    const popup = popups.get(type);
    if(popup && popup.source === source) {
        popup.remove();
    }
}

export function openPopup(
    type: PopupType,
    source: string,
    content: HTMLElement,
    anchorElement: HTMLElement
) {
    if (!popupsContainer) return;
    
    // Create a new popup element if the popup isn't already active
    let element: HTMLDivElement;
    let hadPopup = popups.has(type);
    if(hadPopup) {
        const existingPopup = popups.get(type)!;
        
        // If the open popup is from the same source, close it instead
        if (existingPopup.source === source && existingPopup.open) {
            existingPopup.remove();
            return;
        }
        
        existingPopup.open = true;
        existingPopup.source = source;
        try {
            existingPopup.element.removeChild(existingPopup.element.children[0]);
            existingPopup.element.appendChild(content);
        } catch(e) {
            debugLog("Error updating popup content:", e);
        }
        element = existingPopup.element;
    } else {
        element = document.createElement("div");
        element.appendChild(content);
        popups.set(type, new OpenPopup(type, source, element, null));
    }
    
    popupsContainer.appendChild(element);
    
    const popupsRect = popupsContainer.getBoundingClientRect();
    const anchorRect = anchorElement.getBoundingClientRect();
    const popupRect = element.getBoundingClientRect();

    const anchorCenter = {
        x: anchorRect.left + anchorRect.width / 2,
        y: anchorRect.top + anchorRect.height / 2,
    };
    
    const anchoredLeft = anchorCenter.x - popupRect.width / 2 < popupsRect.left;
    const anchoredRight = anchorCenter.x + popupRect.width / 2 > popupsRect.left + popupsRect.width;
    const anchoredTop = anchorCenter.y - popupRect.height / 2 < popupsRect.top;
    const anchoredBottom = anchorCenter.y + popupRect.height / 2 > popupsRect.top + popupsRect.height;
    
    let left = anchoredLeft || anchoredRight ? null : anchorCenter.x - popupsRect.left;
    let top = anchoredTop || anchoredBottom ? null : anchorCenter.y - popupsRect.top;
    let right = anchoredLeft || anchoredRight ? null : popupsRect.width - (anchorCenter.x - popupsRect.left);
    let bottom = anchoredTop || anchoredBottom ? null : popupsRect.height - (anchorCenter.y - popupsRect.top);

    if(anchoredLeft) left = 0;
    if(anchoredRight) right = 0;
    if(anchoredTop) top = 0;
    if(anchoredBottom) bottom = 0;

    if(!hadPopup) {
        const origin = POPUP_DATA[type].origin;
        const startLeft = origin === PopupOrigin.Left ? (left ?? 0) - popupRect.width : left;
        const startTop = origin === PopupOrigin.Top ? (top ?? 0) - popupRect.height : top;
        const startRight = origin === PopupOrigin.Right ? (right ?? 0) - popupRect.width : right;
        const startBottom = origin === PopupOrigin.Bottom ? (bottom ?? 0) - popupRect.height : bottom;
        
        element.className = "popup";
        element.style.position = "absolute";
        element.style.left = left !== null ? `${startLeft}px` : "";
        element.style.top = top !== null ? `${startTop}px` : "";
        element.style.right = right !== null ? `${startRight}px` : "";
        element.style.bottom = bottom !== null ? `${startBottom}px` : "";
        element.style.opacity = "0";
        
        requestAnimationFrame(() => {
            element.style.left = left !== null ? `${left}px` : "";
            element.style.top = top !== null ? `${top}px` : "";
            element.style.right = right !== null ? `${right}px` : "";
            element.style.bottom = bottom !== null ? `${bottom}px` : "";
            element.style.opacity = "1";
        });
    } else {
        element.style.left = left !== null ? `${left}px` : "";
        element.style.top = top !== null ? `${top}px` : "";
        element.style.right = right !== null ? `${right}px` : "";
        element.style.bottom = bottom !== null ? `${bottom}px` : "";
        element.style.opacity = "1";
    }
    
    updateKeyboardExclusivity();
    
    pollForChanges();
}

function updateKeyboardExclusivity() {
    const requiresExclusivity = Array.from(popups.values()).some(p => p.open && p.requiresExclusiveKeyboard);
    invokePayload<boolean>("set_keyboard_exclusivity", requiresExclusivity);
}

export function initializePopups(cb: DrawCallback) {
    drawCallback = cb;
    updateKeyboardExclusivity();
    
    popupsContainer = document.getElementById("popups") as HTMLDivElement | null;
}

export function updateInputShape(extraRectangles: InputRect[] | null) {
    if(overrideInputShape) return;

    if(extraRectangles === null) {
        pollLoop(lastElapsed);
        return;
    }

    invokePayload<InputRect[]>("set_input_shape", [
        {
            x: 0,
            y: 0,
            width: window.innerWidth,
            height: barThickness,
        },
        {
            x: 0,
            y: window.innerHeight - nonBorderBarThickness,
            width: window.innerWidth,
            // height: nonBorderBarThickness,
            // Expand the height to allow accessing devtools
            height: 1000
        },
        {
            x: 0,
            y: barThickness,
            width: barThickness,
            height: window.innerHeight - barThickness - nonBorderBarThickness,
        },
        {
            x: window.innerWidth - nonBorderBarThickness,
            y: barThickness,
            width: nonBorderBarThickness,
            height: window.innerHeight - barThickness - nonBorderBarThickness,
        },
        ...extraRectangles,
    ]);
}

function pollForChanges() {
    if(polling) return;
    polling = true;

    continueAnimating(true);
    pollLoop(lastElapsed);
}

// This loop should be as low-overhead as possible because it's polling.
// It doesn't run when there are no elements left, however.
function pollLoop(elapsed: number) {
    lastElapsed = elapsed;
    let shouldContinueAnimating = false;
    popupRectangles = [];
    for (const popup of popups.values()) {
        const newBoundingRect = popup.element.getBoundingClientRect();
        const newRect = Rectangle.filledInward(
            newBoundingRect.left - 1,
            newBoundingRect.top - 1,
            newBoundingRect.left + newBoundingRect.width + 1,
            newBoundingRect.top + newBoundingRect.height + 1
        );

        const prevRect = popup.rect;
        if (
            !prevRect ||
            newRect.center.x !== prevRect.center.x ||
            newRect.center.y !== prevRect.center.y ||
            newRect.width !== prevRect.width ||
            newRect.height !== prevRect.height
        ) {
            shouldContinueAnimating = true;
            popup.rect = newRect;
        }

        popupRectangles.push(newRect);
    }
    
    drawCallback?.(elapsed, popupRectangles);
    
    updateInputShape(popupRectangles.map(r => ({
        x: Math.round(r.center.x - r.width / 2),
        y: Math.round(r.center.y - r.height / 2),
        width: Math.round(r.width),
        height: Math.round(r.height)
    } satisfies InputRect)));
    
    if(continueAnimating(shouldContinueAnimating)) {
        requestAnimationFrame(pollLoop);
    } else {
        polling = false;
    }
}
