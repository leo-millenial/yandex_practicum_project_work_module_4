use libloading::{Library, Symbol};
use std::ffi::CString;
use std::path::Path;

use crate::error::PluginError;
use common::BYTES_PER_PIXEL;

/// Plugin function type: returns 0 on success, non-zero on error.
///
/// # Safety
///
/// The function pointer must point to a valid C ABI function with the signature:
/// `unsafe extern "C" fn(u32, u32, *mut u8, *const c_char) -> c_int`
pub type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::ffi::c_char,
) -> std::ffi::c_int;

/// Error codes returned by plugin functions
pub const PLUGIN_SUCCESS: std::ffi::c_int = 0;
pub const PLUGIN_ERROR_NULL_POINTER: std::ffi::c_int = 1;
pub const PLUGIN_ERROR_INVALID_DIMENSIONS: std::ffi::c_int = 2;
pub const PLUGIN_ERROR_PROCESSING: std::ffi::c_int = 3;

pub struct PluginLoader {
    lib: Library,
}

impl PluginLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, PluginError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(PluginError::LoadError(format!(
                "Plugin file does not exist: {}",
                path.display()
            )));
        }

        let lib = unsafe {
            Library::new(path).map_err(|e| {
                PluginError::LoadError(format!(
                    "Failed to load library '{}': {}. This can happen if the library is corrupted or has incompatible ABI.",
                    path.display(),
                    e
                ))
            })?
        };

        Ok(Self { lib })
    }

    pub fn get_process_image_fn(&self) -> Result<ProcessImageFn, PluginError> {
        let symbol: Symbol<ProcessImageFn> = unsafe {
            self.lib
                .get(b"process_image")
                .map_err(|e| {
                    PluginError::SymbolNotFound(format!(
                        "Symbol 'process_image' not found in plugin: {}. \
                        Make sure the plugin exports a function with signature: \
                        unsafe extern \"C\" fn process_image(width: u32, height: u32, rgba_data: *mut u8, params: *const u8)",
                        e
                    ))
                })?
        };

        Ok(*symbol)
    }
}

/// Calls the plugin's `process_image` function with the given parameters.
///
/// # Safety
///
/// * `rgba_data` must be a valid pointer to a mutable slice of exactly
///   `width * height * 4` bytes representing RGBA pixel data.
/// * `params` must be either a valid pointer to a null-terminated UTF-8 string,
///   or null if no parameters are needed.
/// * The plugin function may mutate the RGBA data in place.
pub fn call_plugin_process(
    process_fn: ProcessImageFn,
    width: u32,
    height: u32,
    rgba_data: &mut [u8],
    params: &str,
) -> Result<(), PluginError> {
    let expected_size = (width as usize) * (height as usize) * BYTES_PER_PIXEL;
    if rgba_data.len() != expected_size {
        return Err(PluginError::ProcessingFailed(format!(
            "RGBA data size mismatch: expected {} bytes ({}x{}x4), got {} bytes",
            expected_size,
            width,
            height,
            rgba_data.len()
        )));
    }

    // SAFETY: CString properly creates a null-terminated string required by C plugins.
    // The CString must outlive the pointer used in the FFI call, which is guaranteed
    // since we hold it in this scope and the FFI call completes before CString is dropped.
    let params_cstring = if params.is_empty() {
        None
    } else {
        Some(CString::new(params).map_err(|_| {
            PluginError::ProcessingFailed("Params contain embedded null byte".into())
        })?)
    };

    // Get pointer to the CString's internal buffer, or null if no params
    let params_ptr = params_cstring
        .as_ref()
        .map(|cs| cs.as_ptr())
        .unwrap_or(std::ptr::null());

    tracing::debug!(
        "Calling plugin process_image: {}x{}, params: {:?}",
        width,
        height,
        params
    );

    // SAFETY:
    // - rgba_data.as_mut_ptr() is valid for width*height*4 bytes (validated above)
    // - params_ptr is either null or a valid CString pointer (validated by CString::new)
    // - The plugin function signature matches ProcessImageFn type
    let result = unsafe { process_fn(width, height, rgba_data.as_mut_ptr(), params_ptr) };

    if result != PLUGIN_SUCCESS {
        let error_msg = match result {
            PLUGIN_ERROR_NULL_POINTER => "Plugin received null pointer",
            PLUGIN_ERROR_INVALID_DIMENSIONS => "Plugin reported invalid dimensions",
            PLUGIN_ERROR_PROCESSING => "Plugin processing error",
            _ => "Unknown plugin error",
        };
        return Err(PluginError::ProcessingFailed(format!(
            "{} (error code: {})",
            error_msg, result
        )));
    }

    Ok(())
}
