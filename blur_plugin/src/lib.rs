use std::ffi::CStr;

use common::{BYTES_PER_PIXEL, DEFAULT_BLUR_RADIUS, MAX_BLUR_RADIUS, MIN_BLUR_RADIUS};

/// Processes image by applying a box blur filter.
///
/// # Safety
///
/// * `rgba_data` must be a valid pointer to a slice of `width * height * 4` bytes.
/// * `params` may be null, a valid null-terminated UTF-8 string containing a number,
///   or empty (defaults to radius 3). Radius is clamped to [1, 10].
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
}
