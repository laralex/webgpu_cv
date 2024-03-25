/* tslint:disable */
/* eslint-disable */
/**
*/
export function wasm_startup(): void;
/**
* @param {number} gl
* @param {number} width
* @param {number} height
*/
export function wasm_resize(gl: number, width: number, height: number): void;
/**
* @returns {number}
*/
export function wasm_get_frame_idx(): number;
/**
* @param {string} canvas_element_id
*/
export function wasm_loop(canvas_element_id: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasm_startup: () => void;
  readonly wasm_resize: (a: number, b: number, c: number) => void;
  readonly wasm_get_frame_idx: () => number;
  readonly wasm_loop: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4ac78cd4690e0340: (a: number, b: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
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
export default function __wbg_init (module_or_path: InitInput | Promise<InitInput>): Promise<InitOutput>;
