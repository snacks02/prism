use iced::Color;

fn color_from_image(image: &image::DynamicImage) -> Option<Color> {
    let (mut red, mut green, mut blue, mut weight) = (0u64, 0u64, 0u64, 0u64);
    for pixel in image.to_rgb8().pixels() {
        let [pixel_red, pixel_green, pixel_blue] = pixel.0;
        let maximum = pixel_red.max(pixel_green).max(pixel_blue) as u64;
        let minimum = pixel_red.min(pixel_green).min(pixel_blue) as u64;
        let saturation = (maximum - minimum).saturating_add(1);
        red += pixel_red as u64 * saturation;
        green += pixel_green as u64 * saturation;
        blue += pixel_blue as u64 * saturation;
        weight += saturation;
    }
    Some(Color::from_rgb8(
        (red / weight) as u8,
        (green / weight) as u8,
        (blue / weight) as u8,
    ))
}

pub const COLOR_ACCENT: Color = Color::from_rgba8(84, 127, 182, 1.0);
pub const COLOR_BACKGROUND: Color = Color::from_rgba8(0, 0, 0, 1.0);
pub const COLOR_GRAY_1: Color = Color::from_rgba8(18, 18, 18, 1.0);
pub const COLOR_GRAY_2: Color = Color::from_rgba8(36, 36, 36, 1.0);
pub const COLOR_GRAY_3: Color = Color::from_rgba8(108, 108, 108, 1.0);
pub const COLOR_GRAY_4: Color = Color::from_rgba8(216, 216, 216, 1.0);
pub const ICON_SIZE: u32 = 18;

pub fn accent_color(cover: Option<&image::DynamicImage>) -> Color {
    cover.and_then(color_from_image).unwrap_or(COLOR_ACCENT)
}
