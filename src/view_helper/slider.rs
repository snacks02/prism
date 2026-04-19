use {
    crate::style,
    iced::{
        Border,
        Color,
        Element,
        widget,
    },
    std::ops::RangeInclusive,
};

const RAIL_HEIGHT: f32 = 10.0;

pub fn slider<'a, Message: 'a + Clone>(
    on_change: impl Fn(f32) -> Message + 'a,
    on_release: Message,
    range: RangeInclusive<f32>,
    step: f32,
    value: f32,
) -> Element<'a, Message> {
    let slider = widget::slider(range, value, on_change)
        .height(RAIL_HEIGHT)
        .on_release(on_release)
        .step(step)
        .style(|_, _| widget::slider::Style {
            handle: widget::slider::Handle {
                background: Color::TRANSPARENT.into(),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
                shape: widget::slider::HandleShape::Circle { radius: 0.0 },
            },
            rail: widget::slider::Rail {
                backgrounds: (style::COLOR_GRAY_4.into(), Color::TRANSPARENT.into()),
                border: Border {
                    radius: RAIL_HEIGHT.into(),
                    ..Default::default()
                },
                width: RAIL_HEIGHT,
            },
        });

    widget::container(slider)
        .style(|_| widget::container::Style {
            background: Some(style::COLOR_GRAY_2.into()),
            border: Border {
                radius: f32::MAX.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
