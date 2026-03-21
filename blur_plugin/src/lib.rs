use std::ffi::CStr;

use common::{BYTES_PER_PIXEL, DEFAULT_BLUR_RADIUS, MAX_BLUR_RADIUS, MIN_BLUR_RADIUS};

/// Error codes returned by the plugin
const PLUGIN_SUCCESS: std::ffi::c_int = 0;
const PLUGIN_ERROR_NULL_POINTER: std::ffi::c_int = 1;

/// Processes image by applying a box blur filter.
///
/// # Safety
///
/// * `rgba_data` must be a valid pointer to a slice of `width * height * 4` bytes.
/// * `params` may be null, a valid null-terminated UTF-8 string containing a number,
///   or empty (defaults to radius 3). Radius is clamped to [1, 10].
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
        tracing::error!("Blur plugin: null pointer received");
        return PLUGIN_ERROR_NULL_POINTER;
    }

    if width == 0 || height == 0 {
        tracing::error!("Blur plugin: invalid dimensions {}x{}", width, height);
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

    let radius: usize = params_str
        .trim()
        .parse::<u8>()
        .unwrap_or(DEFAULT_BLUR_RADIUS)
        .clamp(MIN_BLUR_RADIUS, MAX_BLUR_RADIUS) as usize;

    tracing::info!("Blur plugin: {}x{}, radius: {}", w, h, radius);

    let original = rgba_data.to_vec();

    for y in 0..h {
        for x in 0..w {
            let mut r_sum: u32 = 0;
            let mut g_sum: u32 = 0;
            let mut b_sum: u32 = 0;
            let mut a_sum: u32 = 0;
            let mut count: u32 = 0;

            let half_radius = radius / 2;
            for dy in 0..radius {
                for dx in 0..radius {
                    let nx = x + dx;
                    let ny = y + dy;
                    let (nx, ny) = if nx >= half_radius && ny >= half_radius {
                        (nx - half_radius, ny - half_radius)
                    } else {
                        continue;
                    };

                    if nx < w && ny < h {
                        let idx = (ny * w + nx) * BYTES_PER_PIXEL;
                        r_sum += original[idx] as u32;
                        g_sum += original[idx + 1] as u32;
                        b_sum += original[idx + 2] as u32;
                        a_sum += original[idx + 3] as u32;
                        count += 1;
                    }
                }
            }

            let idx = (y * w + x) * BYTES_PER_PIXEL;
            rgba_data[idx] = (r_sum / count) as u8;
            rgba_data[idx + 1] = (g_sum / count) as u8;
            rgba_data[idx + 2] = (b_sum / count) as u8;
            rgba_data[idx + 3] = (a_sum / count) as u8;
        }
    }

    tracing::info!("Blur plugin: complete");
    PLUGIN_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    const BYTES_PER_PIXEL: usize = 4;

    fn create_test_image(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let mut data = Vec::with_capacity((width * height * BYTES_PER_PIXEL as u32) as usize);
        for _ in 0..(width * height) {
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
        data
    }

    #[test]
    fn test_blur_uniform_image_unchanged() {
        let width = 3u32;
        let height = 3u32;
        let mut data = create_test_image(width, height, 100, 150, 200, 255);

        let original = data.clone();
        let result = unsafe { process_image(width, height, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_SUCCESS);
        assert_eq!(data, original);
    }

    #[test]
    fn test_blur_changes_image() {
        let width = 5u32;
        let height = 5u32;
        let mut data = create_test_image(width, height, 0, 0, 0, 255);

        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        data[3] = 255;

        let result = unsafe { process_image(width, height, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_SUCCESS);
        assert_ne!(data[0], 255);
    }

    #[test]
    fn test_blur_with_radius_parameter() {
        let width = 5u32;
        let height = 5u32;
        let mut data = create_test_image(width, height, 50, 100, 150, 255);

        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        data[3] = 255;

        let radius_param = CString::new("2").unwrap();
        let result =
            unsafe { process_image(width, height, data.as_mut_ptr(), radius_param.as_ptr()) };

        assert_eq!(result, PLUGIN_SUCCESS);
    }

    #[test]
    fn test_blur_null_pointer_returns_error() {
        let result = unsafe { process_image(3, 3, std::ptr::null_mut(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_ERROR_NULL_POINTER);
    }

    #[test]
    fn test_blur_zero_dimensions_returns_error() {
        let mut data = vec![0u8; 4];
        let result = unsafe { process_image(0, 3, data.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(result, PLUGIN_ERROR_NULL_POINTER);
    }
}
