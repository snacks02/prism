use {
    crate::style,
    iced::{
        Background,
        Border,
        Color,
        Element,
        widget,
        widget::{
            button::Style,
            center,
            svg,
        },
    },
};

pub fn button<'a, Message: 'a + Clone>(
    background: Background,
    color: Color,
    icon: svg::Handle,
    on_press: Message,
    size: u32,
) -> Element<'a, Message> {
    widget::button(center(
        svg(icon)
            .height(style::ICON_SIZE)
            .style(move |_theme, _status| svg::Style { color: Some(color) })
            .width(style::ICON_SIZE),
    ))
    .height(size)
    .on_press(on_press)
    .padding(0)
    .style(move |_theme, _status| Style {
        background: Some(background),
        border: Border {
            radius: f32::MAX.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .width(size)
    .into()
}
