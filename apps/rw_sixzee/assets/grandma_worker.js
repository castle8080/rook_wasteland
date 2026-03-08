/* grandma_worker.js — loader for the Ask Grandma WASM worker binary.
 *
 * This file is imported as a Web Worker script.  It loads the no-modules
 * wasm-bindgen shim (grandma_worker_core.js) and then calls wasm_bindgen()
 * to initialise the WASM binary.  On failure the error string is posted back
 * to the main thread so the GrandmaPanel can show an error state.
 */
importScripts('./grandma_worker_core.js');
wasm_bindgen('./grandma_worker_core_bg.wasm').catch(function (e) {
    self.postMessage(String(e));
});
