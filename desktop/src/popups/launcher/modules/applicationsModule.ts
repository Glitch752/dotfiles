import { invokePayload } from "../../../utils";
import { Module, ModuleEntry } from "../module";
import { DesktopFile } from "@bindings/DesktopFile";

async function applicationsQuery(query: string): Promise<DesktopFile[]> {
    return await invokePayload<string, DesktopFile[]>("plugin:launcher|applications_query", query)
}

export class ApplicationsModule extends Module {
    static MAX_RESULTS = 8;
    
    constructor() {
        super(3, "Applications", "run-applications");
    }
    
    getActive(query: string): boolean {
        return query.length > 0;
    }
    async getEntries(query: string, _abortSignal: AbortSignal): Promise<ModuleEntry[]> {
        // TODO: Support app Desktop Actions defined in the desktop files, e.g. "New Window" from LibreWolf
        const apps = await applicationsQuery(query);

        return apps
            .slice(0, ApplicationsModule.MAX_RESULTS)
            .map(app => new ModuleEntry(app.name ?? "Unknown application", app.comment, app.icon_path, app.exec ? async () => {
                if(!app.exec) return;

                let executable = app.exec.replace(/%[a-zA-Z]/g, "").trim();
            
                try {
                    invokePayload<string>("plugin:launcher|start_application", executable);
                } catch (e) {
                    console.error("Error starting application", e);
                }
            } : null));
    }
}