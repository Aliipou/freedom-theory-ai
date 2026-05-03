//! C ABI — call Freedom Kernel from any language via JSON.
//!
//! Go, C, Zig, Java (JNA), Node (ffi-napi) — all use the same .so.
//! No Python runtime required.
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::{crypto, engine};
use crate::wire::{VerificationResultWire, VerifyInput};

/// Verify an action against a registry (JSON in → JSON out, signed).
///
/// `input_json`  — UTF-8 JSON: `{"registry": {...}, "action": {...}}`
/// `out_json`    — caller-allocated buffer that receives the result JSON
/// `out_len`     — size of `out_json` in bytes
///
/// Returns 0 on success, -1 on parse/panic error.
/// On error `out_json` contains `{"error":"..."}`.
///
/// Input is limited to 1 MiB to prevent memory exhaustion via crafted JSON.
const MAX_INPUT_BYTES: usize = 1 << 20; // 1 MiB

#[no_mangle]
pub extern "C" fn freedom_kernel_verify(
    input_json: *const c_char,
    out_json: *mut c_char,
    out_len: usize,
) -> i32 {
    let input = unsafe {
        match CStr::from_ptr(input_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                write_buf(out_json, out_len, r#"{"error":"invalid utf-8 in input"}"#);
                return -1;
            }
        }
    };

    if input.len() > MAX_INPUT_BYTES {
        write_buf(out_json, out_len, r#"{"error":"input exceeds 1 MiB limit"}"#);
        return -1;
    }

    let outcome = std::panic::catch_unwind(|| -> Result<String, String> {
        let vi: VerifyInput =
            serde_json::from_str(input).map_err(|e| format!("parse: {e}"))?;
        let mut r = engine::verify(&vi.registry, &vi.action);
        attach_signature(&mut r)?;
        serde_json::to_string(&r).map_err(|e| e.to_string())
    });

    match outcome {
        Ok(Ok(json)) => { write_buf(out_json, out_len, &json); 0 }
        Ok(Err(e))   => { write_buf(out_json, out_len, &format!(r#"{{"error":"{e}"}}"#)); -1 }
        Err(_)       => { write_buf(out_json, out_len, r#"{"error":"kernel panic"}"#); -1 }
    }
}

/// Write the kernel's ed25519 verifying key (hex, 64 chars + NUL) into `out_buf`.
#[no_mangle]
pub extern "C" fn freedom_kernel_pubkey(out_buf: *mut c_char, out_len: usize) -> i32 {
    write_buf(out_buf, out_len, &crypto::pubkey_hex());
    0
}

// ─── internal ────────────────────────────────────────────────────────────────

pub(crate) fn attach_signature(r: &mut VerificationResultWire) -> Result<(), String> {
    let payload = serde_json::to_string(&VerificationResultWire {
        signature: None,
        signing_key: None,
        ..r.clone()
    })
    .map_err(|e| e.to_string())?;
    let (sig, vk) = crypto::sign(payload.as_bytes());
    r.signature = Some(sig);
    r.signing_key = Some(vk);
    Ok(())
}

fn write_buf(buf: *mut c_char, len: usize, s: &str) {
    if len == 0 { return; }
    let bytes = s.as_bytes();
    let n = bytes.len().min(len - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, n);
        *buf.add(n) = 0;
    }
}
