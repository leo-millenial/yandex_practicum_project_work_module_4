use std::ffi::CStr;

use common::BYTES_PER_PIXEL;

/// Error codes returned by the plugin
const PLUGIN_SUCCESS: std::ffi::c_int = 0;
const PLUGIN_ERROR_NULL_POINTER: std::ffi::c_int = 1;

/// Processes image by mirroring it horizontally.
///
/// # Safety
///
/// * `rgba_data` must be a valid pointer to a slice of `width * height * 4` bytes.
/// * `params` may be null or a valid null-terminated UTF-8 string.
///
/// # Returns
///
/// * `0` on success
/// * `1` if `rgba_data` is null
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::ffi::c_char,
) -> std::ffi::c_int {
    if rgba_data.is_null() {
        tracing::error!("Mirror plugin: null pointer received");
        return PLUGIN_ERROR_NULL_POINTER;
    }

    if width == 0 || height == 0 {
        tracing::error!("Mirror plugin: invalid dimensions {}x{}", width, height);
        return PLUGIN_ERROR_NULL_POINTER;
    }

    let w = width as usize;
    let h = height as usize;
    let rgba_data = std::slice::from_raw_parts_mut(rgba_data, w * h * BYTES_PER_PIXEL);

    let params_str = if params.is_null() {
        ""
    } else {
        CStr::from_ptr(params).to_str().unwrap_or_default()
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
    PLUGIN_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, values: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity((width * height * BYTES_PER_PIXEL as u32) as usize);
        for &v in values {
            data.push(v);
            data.push(v);
            data.push(v);
            data.push(255);
        }
        data
    }

    fn pixel_at(data: &[u8], x: usize, y: usize, width: usize) -> [u8; 4] {
        let idx = (y * width + x) * BYTES_PER_PIXEL;
        [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]
    }

    #[test]
    fn test_mirror_horizontal_swap() {
        let width = 4u32;
        let height = 1u32;
        let mut data = create_test_image(width, height, &[1, 2, 3, 4]);

        let left_original = pixel_at(&data, 0, 0, width as usize);
        let right_original = pixel_at(&data, 3, 0, width as usize);

        let result = unsafe { process_image(width, height, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_SUCCESS);
        let left_after = pixel_at(&data, 0, 0, width as usize);
        let right_after = pixel_at(&data, 3, 0, width as usize);
        assert_eq!(left_after, right_original);
        assert_eq!(right_after, left_original);
    }

    #[test]
    fn test_mirror_odd_width_middle_unchanged() {
        let width = 3u32;
        let height = 1u32;
        let mut data = create_test_image(width, height, &[10, 20, 30]);

        let middle_original = pixel_at(&data, 1, 0, width as usize);

        let result = unsafe { process_image(width, height, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_SUCCESS);
        let middle_after = pixel_at(&data, 1, 0, width as usize);
        assert_eq!(middle_after, middle_original);
    }

    #[test]
    fn test_mirror_two_rows() {
        let width = 2u32;
        let height = 2u32;
        let mut data = create_test_image(width, height, &[10, 20, 30, 40]);

        let result = unsafe { process_image(width, height, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_SUCCESS);
        let pixel_0_0 = pixel_at(&data, 0, 0, width as usize);
        let pixel_0_1 = pixel_at(&data, 0, 1, width as usize);
        let pixel_1_0 = pixel_at(&data, 1, 0, width as usize);
        let pixel_1_1 = pixel_at(&data, 1, 1, width as usize);
        assert_eq!(pixel_0_0, [20, 20, 20, 255]);
        assert_eq!(pixel_1_0, [10, 10, 10, 255]);
        assert_eq!(pixel_0_1, [40, 40, 40, 255]);
        assert_eq!(pixel_1_1, [30, 30, 30, 255]);
    }

    #[test]
    fn test_mirror_null_pointer_returns_error() {
        let result = unsafe { process_image(3, 3, std::ptr::null_mut(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_ERROR_NULL_POINTER);
    }

    #[test]
    fn test_mirror_zero_dimensions_returns_error() {
        let mut data = vec![0u8; 4];
        let result = unsafe { process_image(0, 3, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_ERROR_NULL_POINTER);
    }
}
