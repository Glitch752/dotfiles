import { copyToClipboard, invokePayload } from "../../../utils";
import { Module, ModuleEntry } from "../module";
import { RinkResult } from "@bindings/RinkResult";

async function rinkQuery(query: string): Promise<RinkResult> {
    return invokePayload<string, RinkResult>("plugin:launcher|rink_query", query);
}

export class RinkModule extends Module {
    private static RINK_QUERY_PREFIX = "=";
    
    constructor() {
        super("Rink", "run-calculator");
    }
    
    getActive(query: string): boolean {
        return query.startsWith(RinkModule.RINK_QUERY_PREFIX);
    }
    async getEntries(query: string, abortSignal: AbortSignal): Promise<ModuleEntry[]> {
        query = query.slice(RinkModule.RINK_QUERY_PREFIX.length).trim();
        
        try {
            const result = await rinkQuery(query);
            if (abortSignal.aborted) return [];
            
            const error = "Err" in result ? result.Err : null;
            // if(error) return [];
            
            const { title, description, textToCopy } = this.parseRinkOutput(error ? error : (result as any).Ok);
            return [new ModuleEntry(title, description, null, error ? null : () => copyToClipboard(textToCopy))];
        } catch (e) {
            return [new ModuleEntry(`Error running Rink: ${e}`, "", null, null)];
        }
    }
    
    private parseRinkOutput(output: string): { title: string, description: string, textToCopy: string } {
        // Remove newlines and extra spaces
        output = output.replace(/\s+/g, " ").trim();
        
        // If ": " is in the result, it's probably something like "Search result: ...", so we'll split on that
        if (output.includes(": ")) {
            const [title, description] = output.split(": ");
            return { title, description, textToCopy: output };
        }
        
        // Otherwise, if there's something in parentheses at the end, it's likely the units of the result
        const matches = output.match(/^(.*?)(?: \((.*?)\))?$/);
        if (!matches) return { title: output, description: "", textToCopy: output };
        
        const [, title, description] = matches;
        return { title, description: description ?? "", textToCopy: title };
    }
}