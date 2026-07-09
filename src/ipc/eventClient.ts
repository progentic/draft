import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type StopEventListener = UnlistenFn;

/**
 * Registers one raw Tauri event listener. Feature code must subscribe through
 * a typed event wrapper instead of importing Tauri event APIs directly.
 */
export async function listenToEvent(
  eventName: string,
  onPayload: (payload: unknown) => void,
): Promise<StopEventListener> {
  return listen<unknown>(eventName, (event) => onPayload(event.payload));
}
