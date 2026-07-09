import { invoke } from "@tauri-apps/api/core";

type CommandArguments = Record<string, unknown>;

/**
 * Calls one registered Rust command. Feature code must use a typed wrapper
 * instead of importing Tauri APIs directly.
 */
export async function invokeCommand<TResponse>(
  commandName: string,
  commandArguments: CommandArguments,
): Promise<TResponse> {
  return invoke<TResponse>(commandName, commandArguments);
}
