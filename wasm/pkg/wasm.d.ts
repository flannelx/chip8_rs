/* tslint:disable */
/* eslint-disable */
/**
*/
export class WasmChip8 {
  free(): void;
/**
* @param {string} url
* @returns {Promise<WasmChip8>}
*/
  static new(url: string): Promise<WasmChip8>;
/**
* @returns {Uint8Array}
*/
  get_ram(): Uint8Array;
/**
* @returns {Uint8Array}
*/
  get_screen(): Uint8Array;
/**
* @returns {number}
*/
  get_pc(): number;
/**
* @param {Uint32Array} input
* @returns {Promise<void>}
*/
  cycle(input: Uint32Array): Promise<void>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmchip8_free: (a: number) => void;
  readonly wasmchip8_new: (a: number, b: number) => number;
  readonly wasmchip8_get_ram: (a: number, b: number) => void;
  readonly wasmchip8_get_screen: (a: number, b: number) => void;
  readonly wasmchip8_get_pc: (a: number) => number;
  readonly wasmchip8_cycle: (a: number, b: number, c: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb8ed9e370446dd37: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h27891ce45568b150: (a: number, b: number, c: number, d: number) => void;
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
