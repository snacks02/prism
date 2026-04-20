use {
    iced::Color,
    image::DynamicImage,
};

fn color_from_dynamic_image(image: &DynamicImage) -> Color {
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
    Color::from_rgb8(
        (red / weight) as u8,
        (green / weight) as u8,
        (blue / weight) as u8,
    )
}

pub const COLOR_ACCENT: Color = Color::from_rgb8(84, 127, 182);
pub const COLOR_BACKGROUND: Color = Color::from_rgb8(0, 0, 0);
pub const COLOR_GRAY_1: Color = Color::from_rgb8(18, 18, 18);
pub const COLOR_GRAY_2: Color = Color::from_rgb8(36, 36, 36);
pub const COLOR_GRAY_3: Color = Color::from_rgb8(108, 108, 108);
pub const COLOR_GRAY_4: Color = Color::from_rgb8(216, 216, 216);
pub const ICON_SIZE: u32 = 18;

pub fn color_accent(cover: Option<&image::DynamicImage>) -> Color {
    cover.map(color_from_dynamic_image).unwrap_or(COLOR_ACCENT)
}
