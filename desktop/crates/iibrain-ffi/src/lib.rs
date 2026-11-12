//! The core as the phone app loads it (ADR 0011): one C function that
//! takes a request as JSON text and gives the answer as JSON text, and
//! one that frees an answer. What the requests are is the core's
//! `api` module.

// The two C functions are this crate's one unsafe code; each says why
// it is sound.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char};
use std::sync::{Mutex, PoisonError};

use iibrain_core::api::Session;

/// The one core of the app's process. Every isolate of the phone app
/// that loads the library calls into the same one.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Follows one request. A request that waits for a Model does so
/// without holding the session, so other calls go on meanwhile.
fn call(request: &str) -> String {
    let step = {
        let mut session = SESSION.lock().unwrap_or_else(PoisonError::into_inner);
        session.get_or_insert_with(Session::default).begin(request)
    };
    step.finish()
}

/// Takes a request as NUL-terminated UTF-8 JSON and gives the answer
/// the same way. The caller frees the answer with [`iibrain_free`].
/// A null request, or one that is no UTF-8, is answered with an error.
///
/// # Safety
///
/// `request` must be null or point to a NUL-terminated string that
/// stays valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iibrain_call(request: *const c_char) -> *mut c_char {
    let request = if request.is_null() {
        None
    } else {
        // SAFETY: the caller hands in a NUL-terminated string that
        // lives for the call, as the contract above says.
        unsafe { CStr::from_ptr(request) }.to_str().ok()
    };
    let answer = match request {
        Some(request) => std::panic::catch_unwind(|| call(request)).unwrap_or_else(|_| {
            r#"{"error":{"kind":"invalid","message":"the core failed"}}"#.to_string()
        }),
        None => r#"{"error":{"kind":"invalid","message":"no request"}}"#.to_string(),
    };
    // JSON holds no NUL byte; should it ever, the answer is an error.
    CString::new(answer)
        .unwrap_or_else(|_| c"{\"error\":{\"kind\":\"invalid\",\"message\":\"\"}}".to_owned())
        .into_raw()
}

/// Frees an answer [`iibrain_call`] gave.
///
/// # Safety
///
/// `answer` must be null or a pointer [`iibrain_call`] returned that
/// was not freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iibrain_free(answer: *mut c_char) {
    if !answer.is_null() {
        // SAFETY: the pointer came from `CString::into_raw` in
        // `iibrain_call` and is freed once, as the contract above says.
        drop(unsafe { CString::from_raw(answer) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(request: &str) -> String {
        let request = CString::new(request).unwrap_or_default();
        // SAFETY: `request` is a NUL-terminated string that lives for
        // the call; the answer is read before it is freed, once.
        unsafe {
            let answer = iibrain_call(request.as_ptr());
            let text = CStr::from_ptr(answer).to_string_lossy().to_string();
            iibrain_free(answer);
            text
        }
    }

    #[test]
    fn a_request_is_answered_as_json_text() {
        let answer = ask(r#"{"call":"state"}"#);
        assert_eq!(answer, r#"{"ok":{"brain":null}}"#);
    }

    #[test]
    fn what_is_no_request_is_answered_with_an_error() {
        assert!(ask("nonsense").contains(r#""kind":"invalid""#));
        // SAFETY: a null request is allowed; the answer is freed once.
        unsafe {
            let answer = iibrain_call(std::ptr::null());
            assert!(
                CStr::from_ptr(answer)
                    .to_string_lossy()
                    .contains("no request")
            );
            iibrain_free(answer);
            iibrain_free(std::ptr::null_mut());
        }
    }
}
