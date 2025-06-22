export class ModuleEntry {
  constructor(
    public name: string,
    public description: string | null,
    public icon: string | null,
    /** If null, the entry can't be selected. */
    public onActivate: (() => void) | null
  ) { }
}

export abstract class Module {
  constructor(
    /** Higher priority modules are placed higher in the list. */
    public priority: number,
    public name: string,
    public icon: string
  ) {}

  getActive(_query: string): boolean {
    return true;
  }

  /**
   * Get the entries for the given query.  
   * If returning a promise, it must be cancellable with the given AbortSignal.
   */
  abstract getEntries(query: string, abortSignal: AbortSignal): ModuleEntry[] | Promise<ModuleEntry[]>;
}