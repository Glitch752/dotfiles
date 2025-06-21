import { invoke } from "@tauri-apps/api/core";

export function debug_log(text: string) {
  invoke("plugin:bar|debug_log", { msg: text });
}

export function invokePayload<T, R = void>(name: string, payload: T): Promise<R> {
  return invoke(name, { payload });
}