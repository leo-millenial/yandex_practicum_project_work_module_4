use libloading::{Library, Symbol};
use std::path::Path;

use crate::error::PluginError;
use common::BYTES_PER_PIXEL;

pub type ProcessImageFn =
    unsafe extern "C" fn(width: u32, height: u32, rgba_data: *mut u8, params: *const u8);

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

    let params_bytes = params.as_bytes();
    let params_ptr = if params_bytes.is_empty() {
        std::ptr::null()
    } else {
        params_bytes.as_ptr()
    };

    tracing::debug!(
        "Calling plugin process_image: {}x{}, params: {:?}",
        width,
        height,
        params
    );

    unsafe {
        process_fn(width, height, rgba_data.as_mut_ptr(), params_ptr);
    }

    Ok(())
}
