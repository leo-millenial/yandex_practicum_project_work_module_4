use std::ffi::CStr;

use common::BYTES_PER_PIXEL;

/// Processes image by mirroring it horizontally.
///
/// # Safety
///
/// * `rgba_data` must be a valid pointer to a slice of `width * height * 4` bytes.
/// * `params` may be null or a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const u8,
) {
    if rgba_data.is_null() {
        return;
    }

    let w = width as usize;
    let h = height as usize;
    let rgba_data = std::slice::from_raw_parts_mut(rgba_data, w * h * BYTES_PER_PIXEL);

    let params_str = if params.is_null() {
        ""
    } else {
        CStr::from_ptr(params as *const i8)
            .to_str()
            .unwrap_or_default()
    };

    tracing::info!("Mirror plugin: {}x{}, params: {:?}", w, h, params_str);

    for row in 0..h {
        let row_start = row * w * BYTES_PER_PIXEL;

        let mut left = 0usize;
        let mut right = (w - 1) * BYTES_PER_PIXEL;

        while left < right {
            for component in 0..BYTES_PER_PIXEL {
                rgba_data.swap(row_start + left + component, row_start + right + component);
            }
            left += BYTES_PER_PIXEL;
            right -= BYTES_PER_PIXEL;
        }
    }

    tracing::info!("Mirror plugin: complete");
}
