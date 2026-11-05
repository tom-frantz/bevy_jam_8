use bevy::{
    color::{Color, palettes::tailwind},
    ecs::hierarchy::Children,
    scene::{Scene, bsn},
    text::{TextColor as BevyTextColor, TextSpan},
    ui::widget::Text,
};

struct StyledText {
    text: String,
    colour: FancyTextColor,
    // TODO include a font
}

#[derive(Clone, Copy)]
pub enum FancyTextColor {
    DEFAULT,
    HIGHLIGHT,
    R,
    O,
    M,
    G,
    Y,
    N,
    P,
}

impl FancyTextColor {
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "r" => Some(FancyTextColor::R),
            "o" => Some(FancyTextColor::O),
            "m" => Some(FancyTextColor::M),
            "g" => Some(FancyTextColor::G),
            "y" => Some(FancyTextColor::Y),
            "n" => Some(FancyTextColor::N),
            "p" => Some(FancyTextColor::P),
            _ => None,
        }
    }

    pub fn to_bevy_color(&self) -> Color {
        match self {
            FancyTextColor::DEFAULT => Color::srgb(1.0, 1.0, 1.0),
            FancyTextColor::HIGHLIGHT => Color::srgb(0.8, 0.5, 0.2),
            FancyTextColor::R => Color::Srgba(tailwind::RED_500),
            FancyTextColor::O => Color::Srgba(tailwind::ORANGE_400),
            FancyTextColor::M => Color::Srgba(tailwind::SKY_300),
            FancyTextColor::G => Color::Srgba(tailwind::GREEN_600),
            FancyTextColor::Y => Color::Srgba(tailwind::YELLOW_300),
            FancyTextColor::N => Color::Srgba(tailwind::BLUE_500),
            FancyTextColor::P => Color::Srgba(tailwind::PURPLE_500),
        }
    }
}

impl StyledText {
    fn new(text: &str, colour: FancyTextColor) -> Self {
        Self {
            text: text.to_string(),
            colour,
        }
    }
}

fn parse_styled_text(input: &str) -> Vec<StyledText> {
    let mut styled_text = vec![];
    let mut input = input;

    while let Some((plain, after_open)) = input.split_once('<') {
        // before tax is always deault stlyed
        styled_text.push(StyledText::new(plain, FancyTextColor::DEFAULT));

        let (tag, after_tag) = after_open.split_once('>').expect("missing end >");

        let colour = FancyTextColor::from_tag(tag).expect("invalid tag");

        let (coloured, after_close) = after_tag
            .split_once(&format!("<{tag}>"))
            .expect("missing matching end tag");

        // blow up if they trust nest tags
        assert!(
            !coloured.contains('<'),
            "nested tag detected, get outtta here"
        );

        styled_text.push(StyledText::new(coloured, colour));

        input = after_close;
    }

    // final text after last tag also default styling
    styled_text.push(StyledText::new(input, FancyTextColor::DEFAULT));

    styled_text
}

fn text_span_scene(styled: &StyledText) -> impl Scene {
    let text = styled.text.clone();
    let colour = BevyTextColor(styled.colour.to_bevy_color());
    bsn! {
        TextSpan(text)
        colour
    }
}

pub fn render_text_scene(input_string: String) -> impl Scene {
    let spans = parse_styled_text(&input_string)
        .iter()
        .map(text_span_scene)
        .collect::<Vec<_>>();

    bsn! {
        Text
        Children [
            {spans}
        ]
    }
}
