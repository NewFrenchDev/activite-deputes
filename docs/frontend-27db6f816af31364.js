export class IntoUnderlyingByteSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingByteSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get autoAllocateChunkSize() {
        const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingbytesource_cancel(ptr);
    }
    /**
     * @param {ReadableByteStreamController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, controller);
        return ret;
    }
    /**
     * @param {ReadableByteStreamController} controller
     */
    start(controller) {
        wasm.intounderlyingbytesource_start(this.__wbg_ptr, controller);
    }
    /**
     * @returns {ReadableStreamType}
     */
    get type() {
        const ret = wasm.intounderlyingbytesource_type(this.__wbg_ptr);
        return __wbindgen_enum_ReadableStreamType[ret];
    }
}
if (Symbol.dispose) IntoUnderlyingByteSource.prototype[Symbol.dispose] = IntoUnderlyingByteSource.prototype.free;

export class IntoUnderlyingSink {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSinkFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr, 0);
    }
    /**
     * @param {any} reason
     * @returns {Promise<any>}
     */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, reason);
        return ret;
    }
    /**
     * @returns {Promise<any>}
     */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return ret;
    }
    /**
     * @param {any} chunk
     * @returns {Promise<any>}
     */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, chunk);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSink.prototype[Symbol.dispose] = IntoUnderlyingSink.prototype.free;

export class IntoUnderlyingSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr, 0);
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
    /**
     * @param {ReadableStreamDefaultController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, controller);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSource.prototype[Symbol.dispose] = IntoUnderlyingSource.prototype.free;

export function main() {
    wasm.main();
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_4a348b369b009243: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_43c7ccb034739216: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_falsy_3ef25fe1f6d9fbd7: function(arg0) {
            const ret = !arg0;
            return ret;
        },
        __wbg___wbindgen_is_function_18bea6e84080c016: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_c5f5bb76436a9ab1: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_object_8d3fac158b36498d: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_4d5f2c5b2acf65b0: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_4a711ea9d2e1ef93: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_eq_65f99081d9ee8f4d: function(arg0, arg1) {
            const ret = arg0 === arg1;
            return ret;
        },
        __wbg___wbindgen_number_get_eed4462ef92e1bed: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_rethrow_7568bc158db9e1b3: function(arg0) {
            throw arg0;
        },
        __wbg___wbindgen_string_get_d09f733449cbf7a2: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_df03e93053e0f4bc: function(arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            throw new Error(v0);
        },
        __wbg__wbg_cb_unref_9f02ce912168c354: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_addEventListener_3005ac5ed6837415: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.addEventListener(v0, arg3, arg4);
        }, arguments); },
        __wbg_addEventListener_57d995fadad0d0d4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.addEventListener(v0, arg3);
        }, arguments); },
        __wbg_add_872820d59ac4c352: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.add(v0);
        }, arguments); },
        __wbg_altKey_30bab8bfafab5f8e: function(arg0) {
            const ret = arg0.altKey;
            return ret;
        },
        __wbg_appendChild_1e23e55b041fadb7: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_append_4ebb3a86e9c21e3e: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.append(arg1, arg2);
        }, arguments); },
        __wbg_before_b7fd3904fa6181b3: function() { return handleError(function (arg0, arg1) {
            arg0.before(arg1);
        }, arguments); },
        __wbg_body_1f8fe64d6e751875: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_buffer_d8bcb2548b84f613: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_button_afcc53a50febba01: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_byobRequest_d4c119d77083ad7a: function(arg0) {
            const ret = arg0.byobRequest;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_byteLength_9f07c790d3b57e52: function(arg0) {
            const ret = arg0.byteLength;
            return ret;
        },
        __wbg_byteOffset_f141aa06796258e0: function(arg0) {
            const ret = arg0.byteOffset;
            return ret;
        },
        __wbg_call_85e5437fa1ab109d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_call_df7a43aecab856a8: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_cancelBubble_2fa81a3e5d4df275: function(arg0) {
            const ret = arg0.cancelBubble;
            return ret;
        },
        __wbg_checked_3085f9eeea3f0013: function(arg0) {
            const ret = arg0.checked;
            return ret;
        },
        __wbg_childNodes_4e4e5277182da0c0: function(arg0) {
            const ret = arg0.childNodes;
            return ret;
        },
        __wbg_classList_817e1171133ebe82: function(arg0) {
            const ret = arg0.classList;
            return ret;
        },
        __wbg_click_4b94119d468b3078: function(arg0) {
            arg0.click();
        },
        __wbg_cloneNode_43878da34843cf6b: function() { return handleError(function (arg0) {
            const ret = arg0.cloneNode();
            return ret;
        }, arguments); },
        __wbg_close_1d57e0361a94e720: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_close_29dcd39cb851697f: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_composedPath_691ed0976889d507: function(arg0) {
            const ret = arg0.composedPath();
            return ret;
        },
        __wbg_createComment_ab4100c772e7ec9f: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.createComment(v0);
            return ret;
        },
        __wbg_createDocumentFragment_0899325f922e5ed0: function(arg0) {
            const ret = arg0.createDocumentFragment();
            return ret;
        },
        __wbg_createElementNS_fd79b6fb88367352: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            var v1 = getCachedStringFromWasm0(arg3, arg4);
            const ret = arg0.createElementNS(v0, v1);
            return ret;
        }, arguments); },
        __wbg_createElement_d42cc1dfefad50dc: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.createElement(v0);
            return ret;
        }, arguments); },
        __wbg_createObjectURL_0583406d259b32e2: function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_createTask_535c3d20358eb563: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = console.createTask(v0);
            return ret;
        }, arguments); },
        __wbg_createTextNode_e3e72f98b26fa301: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.createTextNode(v0);
            return ret;
        },
        __wbg_ctrlKey_4947d524d8918601: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_decodeURIComponent_5d84a737d465a0b0: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = decodeURIComponent(v0);
            return ret;
        }, arguments); },
        __wbg_decodeURI_56c0c6f04889f861: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = decodeURI(v0);
            return ret;
        }, arguments); },
        __wbg_defaultPrevented_38fbd97937a5db73: function(arg0) {
            const ret = arg0.defaultPrevented;
            return ret;
        },
        __wbg_documentElement_c29770478e190b66: function(arg0) {
            const ret = arg0.documentElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_document_6359a1a8cf0c0ccc: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_done_0ad70482cae88a68: function(arg0) {
            const ret = arg0.done;
            return ret;
        },
        __wbg_enqueue_42644feecc53c816: function() { return handleError(function (arg0, arg1) {
            arg0.enqueue(arg1);
        }, arguments); },
        __wbg_error_51679600615c775d: function(arg0) {
            console.error(arg0);
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
            console.error(v0);
        },
        __wbg_exec_443eef5b50a05fe3: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.exec(v0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_fetch_8508e14acc7090e7: function(arg0, arg1) {
            const ret = arg0.fetch(arg1);
            return ret;
        },
        __wbg_fetch_ec775423198d5d5c: function(arg0, arg1) {
            const ret = arg0.fetch(arg1);
            return ret;
        },
        __wbg_getAttribute_1142eddc52dcf0d9: function(arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg2, arg3);
            const ret = arg1.getAttribute(v0);
            var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len2 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len2, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr2, true);
        },
        __wbg_getDate_c108d9f64ccef1f4: function(arg0) {
            const ret = arg0.getDate();
            return ret;
        },
        __wbg_getElementById_7b7e09c0df99b03f: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.getElementById(v0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getFullYear_334b87dcbcf03db5: function(arg0) {
            const ret = arg0.getFullYear();
            return ret;
        },
        __wbg_getMonth_9871eceda9c5b56a: function(arg0) {
            const ret = arg0.getMonth();
            return ret;
        },
        __wbg_get_00556b8212d195fc: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg2, arg3);
            const ret = arg1.get(v0);
            var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len2 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len2, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr2, true);
        }, arguments); },
        __wbg_get_6f5cf69c8f3f094a: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_c40e2c3262995a8e: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_d0e1306db90b68d9: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_hasAttribute_94976bd1dc5bfc3b: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.hasAttribute(v0);
            return ret;
        },
        __wbg_has_393943c179c58bfe: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.has(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_hash_570b4635c9d99488: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_hash_61da34ef8e5b257a: function(arg0, arg1) {
            const ret = arg1.hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_headers_5960155bb72206d7: function(arg0) {
            const ret = arg0.headers;
            return ret;
        },
        __wbg_history_78c1ef0772e5fa42: function() { return handleError(function (arg0) {
            const ret = arg0.history;
            return ret;
        }, arguments); },
        __wbg_host_7cdba8c50f455650: function(arg0) {
            const ret = arg0.host;
            return ret;
        },
        __wbg_hostname_fd87b671406ed9e6: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.hostname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_href_1422a7ec5e548042: function(arg0, arg1) {
            const ret = arg1.href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_href_3ba20fd3fe994a7b: function(arg0, arg1) {
            const ret = arg1.href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_instanceof_Error_bd90cad2d1c17510: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlAnchorElement_2f67817f31acc840: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLAnchorElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlElement_04782f98385d7019: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Response_4d70bea95d48a514: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ShadowRoot_a4096dd931334181: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ShadowRoot;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_0cc62e4f32542cc4: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_WorkerGlobalScope_602829e223d4907f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof WorkerGlobalScope;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_2efa5973cef6ec32: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_is_69ce89649136abc6: function(arg0, arg1) {
            const ret = Object.is(arg0, arg1);
            return ret;
        },
        __wbg_iterator_e77d2b7575cca5a7: function() {
            const ret = Symbol.iterator;
            return ret;
        },
        __wbg_key_01509308f8cb7840: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_00dd7227fd4626ad: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_3ac0434afa6c5524: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_5e07cf181b2745fb: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_location_733316c688b6f47c: function(arg0) {
            const ret = arg0.location;
            return ret;
        },
        __wbg_message_e65f708c26f11dd3: function(arg0) {
            const ret = arg0.message;
            return ret;
        },
        __wbg_metaKey_077e4cd9ae05ade4: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_name_da14534aff5122dc: function(arg0) {
            const ret = arg0.name;
            return ret;
        },
        __wbg_namespaceURI_79a478448dcfb8b6: function(arg0, arg1) {
            const ret = arg1.namespaceURI;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_new_0_bde4b243a7001c8c: function() {
            const ret = new Date();
            return ret;
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_3ba699f0999bb4ea: function(arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            var v1 = getCachedStringFromWasm0(arg2, arg3);
            const ret = new RegExp(v0, v1);
            return ret;
        },
        __wbg_new_58794ce12509602d: function(arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = new Error(v0);
            return ret;
        },
        __wbg_new_62f131e968c83d75: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_66075f8c2ea6575e: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_9c39b08dfcd79724: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = new URL(v0);
            return ret;
        }, arguments); },
        __wbg_new_a23ce0a01d189235: function() { return handleError(function () {
            const ret = new Headers();
            return ret;
        }, arguments); },
        __wbg_new_fcf4de010e0b2215: function() { return handleError(function () {
            const ret = new URLSearchParams();
            return ret;
        }, arguments); },
        __wbg_new_typed_893dbec5fe999814: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__hd3e5b7e1b6b3d661(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_new_with_base_23aa79ea3733eac5: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            var v1 = getCachedStringFromWasm0(arg2, arg3);
            const ret = new URL(v0, v1);
            return ret;
        }, arguments); },
        __wbg_new_with_byte_offset_and_length_5c494ef1df19d087: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_new_with_str_6ca4c19e2665977a: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = new Request(v0);
            return ret;
        }, arguments); },
        __wbg_new_with_str_and_init_ccd7de5a7b7630b8: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            const ret = new Request(v0, arg2);
            return ret;
        }, arguments); },
        __wbg_new_with_str_sequence_and_options_8de18391ae8030de: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_nextSibling_e82517ad1df7ed9c: function(arg0) {
            const ret = arg0.nextSibling;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_next_5428439dfc1d0362: function() { return handleError(function (arg0) {
            const ret = arg0.next();
            return ret;
        }, arguments); },
        __wbg_next_d314789a105729f3: function(arg0) {
            const ret = arg0.next;
            return ret;
        },
        __wbg_origin_5dc481b643b577be: function(arg0, arg1) {
            const ret = arg1.origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_origin_afa2161e576807fc: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_outerHTML_15343c89c01fe70f: function(arg0, arg1) {
            const ret = arg1.outerHTML;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_parentNode_64c661a3716bb66c: function(arg0) {
            const ret = arg0.parentNode;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_pathname_54730160004370a7: function(arg0, arg1) {
            const ret = arg1.pathname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_pathname_8eea17f7347bc137: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.pathname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_preventDefault_370f245c56eac92e: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_previousSibling_ce85fea5dc035208: function(arg0) {
            const ret = arg0.previousSibling;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_pushState_a50c145997ed7f4d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            var v0 = getCachedStringFromWasm0(arg2, arg3);
            var v1 = getCachedStringFromWasm0(arg4, arg5);
            arg0.pushState(arg1, v0, v1);
        }, arguments); },
        __wbg_push_960865cda81df836: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_querySelector_4e14456d3294d0bc: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            const ret = arg0.querySelector(v0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_queueMicrotask_622e69f0935dfab2: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_d0528786d26e067c: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_removeAttribute_fa111b896cbda960: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.removeAttribute(v0);
        }, arguments); },
        __wbg_removeChild_7a9a93eda7663ae3: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.removeChild(arg1);
            return ret;
        }, arguments); },
        __wbg_removeEventListener_090d4b756985ef27: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.removeEventListener(v0, arg3);
        }, arguments); },
        __wbg_removeProperty_b0fceb051e06c0d6: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            var v0 = getCachedStringFromWasm0(arg2, arg3);
            const ret = arg1.removeProperty(v0);
            const ptr2 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len2, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr2, true);
        }, arguments); },
        __wbg_remove_259623a80f1c4e65: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.remove(v0);
        }, arguments); },
        __wbg_remove_5b13f2ef913fdfb1: function(arg0) {
            arg0.remove();
        },
        __wbg_replaceState_32b7c4324fd9a384: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            var v0 = getCachedStringFromWasm0(arg2, arg3);
            var v1 = getCachedStringFromWasm0(arg4, arg5);
            arg0.replaceState(arg1, v0, v1);
        }, arguments); },
        __wbg_requestAnimationFrame_b2ea44d3667f472c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_resolve_d170483d75a2c8a1: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_respond_0beb7807b2a4bc16: function() { return handleError(function (arg0, arg1) {
            arg0.respond(arg1 >>> 0);
        }, arguments); },
        __wbg_revokeObjectURL_99cd19ae184e2b9d: function() { return handleError(function (arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            URL.revokeObjectURL(v0);
        }, arguments); },
        __wbg_run_059f317bd845fcb7: function(arg0, arg1, arg2) {
            try {
                var state0 = {a: arg1, b: arg2};
                var cb0 = () => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__hb9f6239ca63ef4ea(a, state0.b, );
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = arg0.run(cb0);
                return ret;
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_scrollIntoView_3fe9cc4267b0a807: function(arg0) {
            arg0.scrollIntoView();
        },
        __wbg_scrollTo_b0d16726af992504: function(arg0, arg1, arg2) {
            arg0.scrollTo(arg1, arg2);
        },
        __wbg_searchParams_6b9316dab476f2fe: function(arg0) {
            const ret = arg0.searchParams;
            return ret;
        },
        __wbg_search_15c41efd62733e7e: function(arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_search_5c013caedbd3b575: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_setAttribute_583a391480d5321d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            var v1 = getCachedStringFromWasm0(arg3, arg4);
            arg0.setAttribute(v0, v1);
        }, arguments); },
        __wbg_setProperty_cef7be222c4113f4: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            var v1 = getCachedStringFromWasm0(arg3, arg4);
            arg0.setProperty(v0, v1);
        }, arguments); },
        __wbg_setTimeout_9d56e23573f7138b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_8326741805409e83: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_8540e567c8d5d225: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbg_set_data_a5f0551d45e70291: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.data = v0;
        },
        __wbg_set_download_f9cb96c073ed787d: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.download = v0;
        },
        __wbg_set_headers_e4e56fe005f0b5c9: function(arg0, arg1) {
            arg0.headers = arg1;
        },
        __wbg_set_href_5aafc008633362de: function() { return handleError(function (arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.href = v0;
        }, arguments); },
        __wbg_set_href_c4f90e8cba3a209f: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.href = v0;
        },
        __wbg_set_innerHTML_16484e23f04b9e4f: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.innerHTML = v0;
        },
        __wbg_set_method_e1291768ddb1e35e: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.method = v0;
        },
        __wbg_set_search_dceb4146ee48c532: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.search = v0;
        },
        __wbg_set_textContent_8f86e47ecaa19cf8: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.textContent = v0;
        },
        __wbg_set_type_c5dc1c9ebae88307: function(arg0, arg1, arg2) {
            var v0 = getCachedStringFromWasm0(arg1, arg2);
            arg0.type = v0;
        },
        __wbg_shiftKey_64c9e551b1cbbee1: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_state_260320740dd72ab9: function() { return handleError(function (arg0) {
            const ret = arg0.state;
            return ret;
        }, arguments); },
        __wbg_static_accessor_GLOBAL_THIS_6614f2f4998e3c4c: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_d8e8a2fefe80bc1d: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_e29eaf7c465526b1: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_66e7ca3eef30585a: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_status_3a65028f4384d918: function(arg0) {
            const ret = arg0.status;
            return ret;
        },
        __wbg_stopPropagation_762f8eeea38d59ce: function(arg0) {
            arg0.stopPropagation();
        },
        __wbg_style_5b52b97fc5c5a29c: function(arg0) {
            const ret = arg0.style;
            return ret;
        },
        __wbg_target_e438fc382e24227f: function(arg0, arg1) {
            const ret = arg1.target;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_target_f82e801b73cc6490: function(arg0) {
            const ret = arg0.target;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_textContent_c4d59dbe1d8a49cd: function(arg0, arg1) {
            const ret = arg1.textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_text_96bbece7b1823162: function() { return handleError(function (arg0) {
            const ret = arg0.text();
            return ret;
        }, arguments); },
        __wbg_then_1170ade08ea65bc7: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_fdc17de424bf508a: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_toString_a22faf60361782a5: function(arg0) {
            const ret = arg0.toString();
            return ret;
        },
        __wbg_toString_d5259ad3c18c4923: function(arg0) {
            const ret = arg0.toString();
            return ret;
        },
        __wbg_url_82c68d7e97d56df4: function(arg0, arg1) {
            const ret = arg1.url;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_value_414b42ce7b3eca22: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbg_value_d6565ff085b86cd6: function(arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_view_3120c63593e25b87: function(arg0) {
            const ret = arg0.view;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_warn_52ab87a85aca283f: function(arg0) {
            console.warn(arg0);
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1393, function: Function { arguments: [], shim_idx: 1391, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h7eeb34549251b2a3, wasm_bindgen__convert__closures_____invoke__h472a22742121bec6);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1394, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1392, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h083c9dd52a621674, wasm_bindgen__convert__closures_____invoke__h71f060f7debb04ef);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1468, function: Function { arguments: [], shim_idx: 1443, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h24dd7bcc8c03b917, wasm_bindgen__convert__closures_____invoke__hedd061e6ff42e509);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1503, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1504, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hd4584e47298a8336, wasm_bindgen__convert__closures_____invoke__he42065b2d4eee644);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1505, function: Function { arguments: [], shim_idx: 1506, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h6728197c04b74950, wasm_bindgen__convert__closures_____invoke__ha04b8784e42cc66d);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1514, function: Function { arguments: [Externref], shim_idx: 1559, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hdefcdf59e2f209de, wasm_bindgen__convert__closures_____invoke__ha242315af1733bd6);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 659, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 765, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h9c4586280a500152, wasm_bindgen__convert__closures_____invoke__h9ee8a9d745598846);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 660, function: Function { arguments: [NamedExternref("ErrorEvent")], shim_idx: 766, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__haab49f31fc595c62, wasm_bindgen__convert__closures_____invoke__h7de6c8dbb80ba9b2);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 661, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 767, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h9b239c056b94757c, wasm_bindgen__convert__closures_____invoke__h6a8e30c858343403);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 662, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 764, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h5a674b5cde09eabf, wasm_bindgen__convert__closures_____invoke__h53f66104218b641d);
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            var v0 = getCachedStringFromWasm0(arg0, arg1);
            // Cast intrinsic for `Ref(CachedString) -> Externref`.
            const ret = v0;
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./frontend_bg.js": import0,
    };
}

function wasm_bindgen__convert__closures_____invoke__h472a22742121bec6(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h472a22742121bec6(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hedd061e6ff42e509(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__hedd061e6ff42e509(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__ha04b8784e42cc66d(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__ha04b8784e42cc66d(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hb9f6239ca63ef4ea(arg0, arg1) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__hb9f6239ca63ef4ea(arg0, arg1);
    return ret !== 0;
}

function wasm_bindgen__convert__closures_____invoke__h71f060f7debb04ef(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h71f060f7debb04ef(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__he42065b2d4eee644(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__he42065b2d4eee644(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h9ee8a9d745598846(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h9ee8a9d745598846(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h7de6c8dbb80ba9b2(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7de6c8dbb80ba9b2(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h6a8e30c858343403(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h6a8e30c858343403(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h53f66104218b641d(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h53f66104218b641d(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__ha242315af1733bd6(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__ha242315af1733bd6(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__hd3e5b7e1b6b3d661(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__hd3e5b7e1b6b3d661(arg0, arg1, arg2, arg3);
}


const __wbindgen_enum_ReadableStreamType = ["bytes"];
const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0, 1));
const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0, 1));
const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getCachedStringFromWasm0(ptr, len) {
    if (ptr === 0) {
        return getFromExternrefTable0(len);
    } else {
        return getStringFromWasm0(ptr, len);
    }
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getFromExternrefTable0(idx) { return wasm.__wbindgen_externrefs.get(idx); }

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('frontend_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
