use serenity::builder::CreateInteractionResponseData;
use crate::consts::ERROR_COLOR;
use super::InteractionRender;

#[derive(Clone)]
pub struct NotYourTurnRender;
impl InteractionRender for NotYourTurnRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		msg
		.ephemeral(true)
		.embed(|e| e
			.description("**[**It is not your turn.**]**")
			.color(ERROR_COLOR))
	}
}

#[derive(Clone)]
pub struct NotInvolvedRender;
impl InteractionRender for NotInvolvedRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		msg
		.ephemeral(true)
		.embed(|e| e
			.description("**[**You're not involved in this.**]**")
			.color(ERROR_COLOR))
	}
}
