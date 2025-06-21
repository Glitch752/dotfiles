import { invoke } from "@tauri-apps/api/core";

export function debugLog(...values: any[]) {
    const text = values.map(v => {
        if(typeof v === "string") return v;
        return JSON.stringify(v);
    }).join(" ");
    invoke("plugin:bar|debug_log", { msg: text });
}

export function invokePayload<T, R = void>(name: string, payload: T): Promise<R> {
    return invoke(name, { payload });
}

export function fallingEdgeDebouncer(delay: number): ((value: boolean) => boolean) {
    let lastTrueTime = Date.now();
    let lastValue = false;
    
    return (value: boolean): boolean => {
        const now = Date.now();
        if (value) {
            lastTrueTime = now;
            lastValue = true;
            return true;
        } else if (now - lastTrueTime >= delay) {
            lastValue = false;
            return false;
        }
        return lastValue;
    };
}

export function debounceFunc(delay: number, func: (() => void)): () => void {
    let timeoutId: number | null = null;

    return () => {
        if (timeoutId !== null) {
            clearTimeout(timeoutId);
        }
        timeoutId = window.setTimeout(() => {
            func();
            timeoutId = null;
        }, delay);
    };
}

export function debounceAsyncFunc<A, T extends Array<A>, R>(delay: number, func: (...args: T) => Promise<R>): (...args: T) => Promise<R> {
    let timeoutId: number | null = null;

    return (...args: T): Promise<R> => {
        return new Promise((resolve, reject) => {
            if (timeoutId !== null) {
                clearTimeout(timeoutId);
            }
            timeoutId = window.setTimeout(() => {
                func(...args).then(resolve).catch(reject);
                timeoutId = null;
            }, delay);
        });
    };
}

export function copyToClipboard(text: string) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
        return navigator.clipboard.writeText(text);
    }
}

/** A simple observable state container */
export class Observable<T> {
    private _value: T;
    private listeners: ((value: T) => void)[] = [];

    constructor(initialValue: T) {
        this._value = initialValue;
    }

    get(): T {
        return this._value;
    }

    set(newValue: T): void {
        if (this._value !== newValue) {
            this._value = newValue;
            this.listeners.forEach(l => l(newValue));
        }
    }

    subscribe(listener: (value: T) => void): void {
        this.listeners.push(listener);
    }
}