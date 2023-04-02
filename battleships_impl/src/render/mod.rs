use serenity::builder::{CreateMessage, CreateInteractionResponseData, CreateInteractionResponseFollowup};

// Define the sub-modules
mod start;
mod place;
mod next_turns;
mod fire;
mod wrong;
pub(crate) mod utility;

// Selectively re-export the sub-modules
pub use start::StartRender;
pub use place::{NextPlaceRender, PlaceRender};
pub use next_turns::{FirstTurnRender, NextTurnRender, NextTurnInfo};
pub use fire::{FireRender, ChooseFireRender, InvalidFireRender, InvalidFireReason};
pub use wrong::{NotYourTurnRender, NotInvolvedRender};
pub use utility::renders::RemoveButtonsRender;

pub trait InteractionRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a>;
}

pub trait FollowUpRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a>;
}

pub trait MessageRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>;
}
