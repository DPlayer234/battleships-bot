use serenity::utils::Color;

pub type SerenityError = serenity::Error;
pub type SerenityResult<T = ()> = Result<T, SerenityError>;

pub const CUSTOM_ID_PREFIX: &str = "#bs#";
pub const EMBED_COLOR: Color = Color::BLITZ_BLUE;
pub const RETRY_COLOR: Color = Color::ORANGE;
pub const ERROR_COLOR: Color = Color::DARK_RED;