import { copyToClipboard, invokePayload } from "../../../utils";
import { Module, ModuleEntry } from "../module";
import { Symbol } from "@bindings/Symbol";

async function symbolsQuery(query: string): Promise<Symbol[]> {
    return await invokePayload<string, Symbol[]>("plugin:launcher|symbols_query", query)
}

export class SymbolsModule extends Module {
    static SYMBOLS_PREFIX = "";
    static MAXIMUM_RESULTS = 3;
    
    constructor() {
        super(0, "Symbols", "run-symbols");
    }
    
    getActive(query: string): boolean {
        return query.startsWith(SymbolsModule.SYMBOLS_PREFIX) && query.length > SymbolsModule.SYMBOLS_PREFIX.length;
    }
    async getEntries(query: string, abortSignal: AbortSignal): Promise<ModuleEntry[]> {
        query = query.slice(SymbolsModule.SYMBOLS_PREFIX.length).trim();
        
        const result = await symbolsQuery(query);
        if(abortSignal.aborted) return [];
        return result.slice(0, SymbolsModule.MAXIMUM_RESULTS).map(({ name, value }) => new ModuleEntry(value, name, null, () => {
            copyToClipboard(value);
        }));
    }
}