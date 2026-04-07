use {
    crate::style,
    iced::{
        Color,
        widget,
        widget::{
            Button,
            center,
            svg,
        },
    },
};

pub fn button<'a, Message: 'a>(color: Color, icon: svg::Handle, size: u32) -> Button<'a, Message> {
    widget::button(center(
        svg(icon)
            .height(style::ICON_SIZE)
            .style(move |_theme, _status| svg::Style { color: Some(color) })
            .width(style::ICON_SIZE),
    ))
    .height(size)
    .padding(0)
    .style(|theme, status| widget::button::Style {
        background: Some(Color::TRANSPARENT.into()),
        border: Default::default(),
        ..widget::button::primary(theme, status)
    })
    .width(size)
}
