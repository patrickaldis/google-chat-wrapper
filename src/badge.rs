use ab_glyph::{FontArc, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_text_mut, text_size};

/// The base application icon (128x128 RGBA PNG).
const BASE_ICON: &[u8] = include_bytes!("../icons/icon.png");

/// Font used for rendering the badge count.
const FONT_DATA: &[u8] = include_bytes!("DejaVuSans-Bold.ttf");

const BADGE_COLOR: Rgba<u8> = Rgba([255, 40, 40, 255]);
const TEXT_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);

/// Return the base icon as an `RgbaImage`.
fn load_base_icon() -> RgbaImage {
    image::load_from_memory_with_format(BASE_ICON, image::ImageFormat::Png)
        .expect("embedded base icon is invalid")
        .to_rgba8()
}

/// Raw rendered icon data (RGBA pixels).
pub struct RenderedIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl RenderedIcon {
    /// Convert to a ksni Icon (ARGB32, network byte order).
    pub fn to_ksni_icon(&self) -> ksni::Icon {
        let mut argb = self.rgba.clone();
        // Rotate each pixel from RGBA to ARGB.
        for pixel in argb.chunks_exact_mut(4) {
            pixel.rotate_right(1);
        }
        ksni::Icon {
            width: self.width as i32,
            height: self.height as i32,
            data: argb,
        }
    }
}

/// Render the application icon, optionally with an unread-count badge in the
/// bottom-right corner.
///
/// - `count == 0`: returns the plain base icon.
/// - `count > 0`:  returns the base icon with a red circle + white number.
/// - counts above 99 are displayed as "99+".
pub fn render(count: u32) -> RenderedIcon {
    let mut icon = load_base_icon();
    let (width, height) = (icon.width(), icon.height());

    if count > 0 {
        draw_badge(&mut icon, count);
    }

    RenderedIcon {
        rgba: icon.into_raw(),
        width,
        height,
    }
}

/// Draw a red badge circle with a white count in the bottom-right corner.
fn draw_badge(icon: &mut RgbaImage, count: u32) {
    let label = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    let font = FontArc::try_from_slice(FONT_DATA).expect("embedded font is invalid");

    // Scale the font relative to icon size. For a 128px icon, a scale of ~48
    // gives a nicely readable single-digit number.
    let icon_size = icon.width() as f32;
    let scale = PxScale::from(icon_size * 0.65);

    let (text_w, text_h) = text_size(scale, &font, &label);

    // Badge circle: sized to fit the text with some padding.
    let padding = (icon_size * 0.08) as i32;
    let radius = (text_w.max(text_h) as i32 / 2) + padding;

    // Position the badge in the bottom-right corner, inset by the radius so
    // the entire circle stays within the icon bounds.
    let cx = icon.width() as i32 - radius - 1;
    let cy = icon.height() as i32 - radius - 1;

    draw_filled_circle_mut(icon, (cx, cy), radius, BADGE_COLOR);

    // Center the text inside the circle.
    let text_x = cx - text_w as i32 / 2;
    let text_y = cy - text_h as i32 / 2;

    draw_text_mut(icon, TEXT_COLOR, text_x, text_y, scale, &font, &label);
}
