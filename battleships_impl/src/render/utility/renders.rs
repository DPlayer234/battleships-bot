use serenity::builder::{CreateMessage, CreateInteractionResponseData, CreateInteractionResponseFollowup, CreateComponents};
use serenity::model::prelude::component::ButtonStyle;
use battleships_model::game_state::*;

use crate::action::{GameAction, GameActionKind};

use super::super::{MessageRender, InteractionRender, FollowUpRender};

#[derive(Clone)]
pub struct RemoveButtonsRender;
impl RemoveButtonsRender {
	fn remove_components(components: &mut CreateComponents) -> &mut CreateComponents {
		components
		.create_action_row(|r| r
			.create_button(|b| b
				.custom_id("--never")
				.label("...")
				.style(ButtonStyle::Secondary)
				.disabled(true)))
	}
}

impl InteractionRender for RemoveButtonsRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		msg.components(Self::remove_components)
	}
}

impl MessageRender for RemoveButtonsRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		msg.components(Self::remove_components)
	}
}

#[derive(Clone)]
pub struct SharedPrepareRender(pub GameState, pub GameActionKind);
impl SharedPrepareRender {
	fn create_components(self, components: &mut CreateComponents) -> &mut CreateComponents {
		let button_id = GameAction::new(self.1, self.0).to_id();
		
		components
		.create_action_row(|r| r
			.create_button(|b| b
				.custom_id(button_id)
				.label("Prepare")
				.style(ButtonStyle::Primary)))
	}
}

impl MessageRender for SharedPrepareRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		msg.components(|c| self.create_components(c))
	}
}

impl FollowUpRender for SharedPrepareRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		msg.components(|c| self.create_components(c))
	}
}