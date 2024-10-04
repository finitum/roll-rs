/* tslint:disable */
/* eslint-disable */
/**
*/
export function init_console(): void;
/**
* @param {string} s
* @param {boolean} advanced
* @returns {string}
*/
export function roll_dice_short(s: string, advanced: boolean): string;
/**
* @param {string} s
* @param {boolean} advanced
* @returns {any}
*/
export function roll_dice(s: string, advanced: boolean): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly roll_dice_short: (a: number, b: number, c: number, d: number) => void;
  readonly roll_dice: (a: number, b: number, c: number, d: number) => void;
  readonly init_console: () => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
