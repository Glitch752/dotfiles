import { convertFileSrc } from "@tauri-apps/api/core";
import { debounceFunc, Observable } from "../../utils";
import { closeLauncher } from "../launcher";
import { Module, ModuleEntry } from "./module";
import { ApplicationsModule } from "./modules/applicationsModule";
import { DictionaryModule } from "./modules/dictionaryModule";
import { RinkModule } from "./modules/rinkModule";
import { SymbolsModule } from "./modules/symbolsModule";
import { WebSearchModule } from "./modules/webSearchModule";

type MaybePromise<T> = T | Promise<T>;
type ModuleResult = {
    module: Module,
    entries: ModuleEntry[]
};

/**
 * Returns module results that are currently active.
 * If a module returns a promise, it will be resolved and added to the results.
 * All promises must not resolve after the abort signal is fired.
 */
function getModuleResults(modules: Module[], query: string, abortSignal: AbortSignal): MaybePromise<ModuleResult>[] {
    let results: MaybePromise<ModuleResult>[] = [];
    for (const module of modules) {
        if (module.getActive(query)) {
            const entries = module.getEntries(query, abortSignal);
            const createResult = (resolvedEntries: ModuleEntry[]): ModuleResult => ({
                module: module,
                entries: resolvedEntries
            });

            if (entries instanceof Promise) {
                // Ensure promises that are aborted don't cause unhandled rejection errors
                const safePromise = entries.catch(err => {
                    // AbortErrors are expected, other errors should be logged.
                    if (err.name !== 'AbortError') {
                        console.error(`Module ${module.name} failed:`, err);
                    }
                    return []; // Return empty entries on error/abort
                });
                results.push(safePromise.then(createResult));
            } else if (entries.length > 0) {
                results.push(createResult(entries));
            }
        }
    }
    return results;
}

export default function addLauncherPopup(content: HTMLDivElement) {
    let currentAbortController = new AbortController();
    const modules: Module[] = [
        new ApplicationsModule(),
        new DictionaryModule(),
        new RinkModule(),
        new WebSearchModule(),
        new SymbolsModule()
    ];
    
    const query = new Observable("");
    // resolvedResults holds only successfully loaded module results
    const resolvedResults = new Observable<ModuleResult[]>([]);
    // A unique ID of the highlighted entry
    const highlightedEntryId = new Observable<string | null>(null);

    const entryInput = document.createElement("input");
    entryInput.type = "text";

    const resultsBox = document.createElement("div");
    resultsBox.className = "results";

    const resultWrapper = document.createElement("div");
    resultWrapper.className = "results-wrapper";
    resultWrapper.appendChild(resultsBox);

    content.append(entryInput, resultWrapper);

    const getEntryId = (moduleName: string, entryName: string) => `${moduleName}::${entryName}`;

    const getActivatableItems = () => {
        return resolvedResults.get().flatMap(result =>
            result.entries
                .filter(entry => entry.onActivate)
                .map(entry => ({
                    id: getEntryId(result.module.name, entry.name),
                    entry,
                    module: result.module
                }))
        );
    };
    
    const getHighlightedEntry = () => {
        const id = highlightedEntryId.get();
        if (!id) return null;
        return getActivatableItems().find(item => item.id === id)?.entry || null;
    };

    const navigateHighlight = (direction: 'next' | 'previous') => {
        const activatable = getActivatableItems();
        if (activatable.length === 0) return;

        const currentId = highlightedEntryId.get();
        const currentIndex = currentId ? activatable.findIndex(item => item.id === currentId) : -1;
        
        let nextIndex: number;
        if (direction === 'next') {
            nextIndex = (currentIndex + 1) % activatable.length;
        } else {
            nextIndex = currentIndex <= 0 ? activatable.length - 1 : currentIndex - 1;
        }

        highlightedEntryId.set(activatable[nextIndex].id);
    };
    const previousHighlight = () => navigateHighlight('previous');
    const nextHighlight = () => navigateHighlight('next');

    function createEntryElement(entry: ModuleEntry, module: Module): HTMLButtonElement {
        const button = document.createElement("button");
        button.className = "entry";
        // Use a stable data-attribute to identify the element
        button.dataset.entryId = getEntryId(module.name, entry.name);

        if (entry.onActivate) {
            button.onclick = () => {
                entry.onActivate!();
                closeLauncher();
            };
        } else {
            button.disabled = true;
        }

        const entryContent = document.createElement("div");
        entryContent.className = "entry-content";

        const entryIcon = document.createElement("img");
        entryIcon.className = "icon";
        entryIcon.src = entry.icon ? convertFileSrc(entry.icon) : "";
        entryIcon.alt = entry.name;
        entryIcon.style.visibility = entry.icon ? "visible" : "hidden";

        const textBox = document.createElement("div");
        textBox.className = "text";

        const nameLabel = document.createElement("i");
        nameLabel.className = "name";
        nameLabel.textContent = entry.name;
        textBox.appendChild(nameLabel);

        const descriptionLabel = document.createElement("div");
        descriptionLabel.className = "description";
        descriptionLabel.style.display = entry.description ? "block" : "none";
        if (entry.description) {
            descriptionLabel.textContent = entry.description;
        }
        textBox.appendChild(descriptionLabel);

        entryContent.append(entryIcon, textBox);
        button.appendChild(entryContent);
        return button;
    }

    // Map from module name to results for diffing
    let previousModuleResults: Map<string, number> = new Map();
    const render = () => {
        const results = resolvedResults.get();
        const highlightedId = highlightedEntryId.get();
        
        let statusLabel = resultsBox.querySelector<HTMLSpanElement>(".status-label");
        if (!statusLabel) {
            statusLabel = document.createElement("span");
            statusLabel.className = "status-label";
            resultsBox.prepend(statusLabel);
        }

        // If the results/modules present changed (and not just their content/highlight),
        // rebuild the tree. Otherwise, just update it.
        const currentModuleResults = new Map<string, number>();
        results.forEach(({ module, entries }) => {
            currentModuleResults.set(module.name, entries.length);
        });
        const resultsChanged = previousModuleResults.size !== currentModuleResults.size ||
            Array.from(previousModuleResults.entries()).some(([name, count]) => currentModuleResults.get(name) !== count);
        
            previousModuleResults = currentModuleResults;
        if(resultsChanged) {
            resultsBox.innerHTML = "";
        }

        const hasResults = results.some(r => r.entries.length > 0);

        if (hasResults) {
            statusLabel.style.display = 'none';
        } else {
            statusLabel.style.display = 'block';
        }
        
        const presentModuleNames = new Set(results.map(r => r.module.name));
        
        // Remove modules that no longer have results
        for (const moduleEl of Array.from(resultsBox.querySelectorAll<HTMLDivElement>(".module"))) {
            if (!presentModuleNames.has(moduleEl.dataset.moduleName!)) {
                moduleEl.remove();
            }
        }

        // Update and add modules/entries
        results.forEach(({ module, entries }) => {
            let moduleBox = resultsBox.querySelector<HTMLDivElement>(`[data-module-name="${module.name}"]`);

            if (!moduleBox) {
                moduleBox = document.createElement("div");
                moduleBox.className = "module";
                moduleBox.dataset.moduleName = module.name;

                const moduleHeader = document.createElement("div");
                moduleHeader.className = "module-header";
                const icon = document.createElement("i");
                icon.className = "icon";
                icon.textContent = `[${module.icon.substring(0, 1)}]`;
                const label = document.createElement("span");
                label.textContent = module.name;
                moduleHeader.append(icon, label);

                const entriesBox = document.createElement("div");
                entriesBox.className = "entries";
                
                moduleBox.append(moduleHeader, entriesBox);
                resultsBox.appendChild(moduleBox);
            }

            const entriesBox = moduleBox.querySelector<HTMLDivElement>(".entries")!;
            const currentEntryIds = new Set(entries.map(e => getEntryId(module.name, e.name)));

            // Remove old entries that are no longer present
            for (const entryEl of Array.from(entriesBox.children)) {
                if (!currentEntryIds.has((entryEl as HTMLElement).dataset.entryId!)) {
                    entryEl.remove();
                }
            }

            // Update and add new entries
            entries.forEach(entry => {
                const entryId = getEntryId(module.name, entry.name);
                let entryEl = entriesBox.querySelector<HTMLButtonElement>(`[data-entry-id="${CSS.escape(entryId)}"]`);

                if (!entryEl) {
                    entryEl = createEntryElement(entry, module);
                    entriesBox.appendChild(entryEl);
                }
                
                entryEl.classList.toggle("highlighted", entryId === highlightedId);
                const descEl = entryEl.querySelector<HTMLDivElement>(".description")!;
                descEl.textContent = entry.description || "";
                descEl.style.display = entry.description ? "block" : "none";
            });
        });
        
        if(resultsChanged) {
            resultsBox.animate([
                { transform: "scale(1)", opacity: 1 },
                { transform: "scale(0.95)", opacity: 0.8 },
                { transform: "scale(1)", opacity: 1 }
            ], { duration: 150, easing: 'cubic-bezier(0.25, 0.1, 0.25, 1)' });
        }

        resultWrapper.style.height = `${resultsBox.scrollHeight}px`;
    };
    const debouncedRender = debounceFunc(50, render);

    query.subscribe(q => {
        currentAbortController.abort();
        currentAbortController = new AbortController();
        const results = getModuleResults(modules, q, currentAbortController.signal);
        
        const initialSyncResults: ModuleResult[] = [];
        const promises: Promise<ModuleResult>[] = [];
        results.forEach(r => (r instanceof Promise ? promises.push(r) : initialSyncResults.push(r)));

        // Include the previous module results for modules that are still present in the initial sync results
        const previousResults = resolvedResults.get();
        resolvedResults.set([...initialSyncResults, ...previousResults.filter(r => r.module.getActive(q))]);
        
        // Auto-highlight the first item from the synchronous results
        highlightedEntryId.set(getActivatableItems()[0]?.id || null);

        promises.forEach(promise => {
            promise.then(newlyResolved => {
                if (currentAbortController.signal.aborted) return;

                if (!newlyResolved) return;

                const currentResults = resolvedResults.get();
                const currentResultsWithoutModule = currentResults.filter(r => r.module.name !== newlyResolved.module.name);

                if (newlyResolved.entries.length === 0) {
                    resolvedResults.set(currentResultsWithoutModule);
                    return;
                }

                resolvedResults.set([...currentResultsWithoutModule, newlyResolved].sort(
                    (a, b) => b.module.priority - a.module.priority
                ));

                // If nothing was highlighted before, highlight the new first item.
                if (!highlightedEntryId.get()) {
                    highlightedEntryId.set(getActivatableItems()[0]?.id || null);
                }
            });
        });
    });

    // Re-render whenever the data or highlight changes
    resolvedResults.subscribe(debouncedRender);
    highlightedEntryId.subscribe(debouncedRender);
    
    entryInput.addEventListener("input", () => {
        highlightedEntryId.set(null); 
        query.set(entryInput.value);
    });

    entryInput.addEventListener("keydown", (e) => {
        let handled = false;
        if (e.key === "Tab" || e.key === "ArrowDown" || e.key === "ArrowUp") {
            if (e.key === "Tab" && e.shiftKey || e.key === "ArrowUp") {
                previousHighlight();
            } else {
                nextHighlight();
            }
            handled = true;
        } else if (e.key === "Enter") {
            const entry = getHighlightedEntry();
            if (entry?.onActivate) {
                entry.onActivate();
                closeLauncher();
            }
            handled = true;
        } else if (e.key === "Escape") {
            closeLauncher();
            handled = true;
        }

        if (handled) {
            e.preventDefault();
        }
    });

    setTimeout(() => entryInput.focus(), 0);
    query.set(""); // Trigger an initial empty query to load default results
}