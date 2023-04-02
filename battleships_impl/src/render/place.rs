use serenity::builder::{CreateEmbed, CreateInteractionResponseData, CreateInteractionResponseFollowup, CreateComponents};
use serenity::model::prelude::*;
use serenity::model::prelude::component::ButtonStyle;
use serenity::utils::*;
use battleships_model::game_state::*;

use crate::consts::EMBED_COLOR;
use crate::action::{GameAction, GameActionKind};

use super::{FollowUpRender, InteractionRender};
use super::utility::render_target::*;
use super::utility::renders::SharedPrepareRender;

#[derive(Clone)]
pub struct NextPlaceRender(pub GameState);
impl FollowUpRender for NextPlaceRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		let mut desc = MessageBuilder::new();
		desc.push_bold('[')
			.mention(&UserId(self.0.current().user_id))
			.push(", prepare as well!")
			.push_bold(']');

		SharedPrepareRender(self.0, GameActionKind::Place)
		.render_follow_up(msg)
		.ephemeral(false)
		.embed(|e| e
			.description(desc)
			.color(EMBED_COLOR))
	}
}

#[derive(Clone)]
pub struct PlaceRender(pub GameState);
impl PlaceRender {
	fn create_embed<'a>(&'_ self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
		let player = self.0.current();

		let mut buffer = RenderTarget::new();
		buffer.set_ships(player);

		embed
		.description(buffer.render_grid("[Okay?]", &Emotes::OWN))
		.color(EMBED_COLOR)
	}

	fn create_components(self, components: &mut CreateComponents) -> &mut CreateComponents {
		let random_button_id = GameAction::new(GameActionKind::RandomizePlace, self.0.clone()).to_id();
		let confirm_button_id = GameAction::new(GameActionKind::ConfirmPlace, self.0).to_id();

		components
		.create_action_row(|r| r
			.create_button(|b| b
				.custom_id(confirm_button_id)
				.label("Confirm")
				.style(ButtonStyle::Success))
			.create_button(|b| b
				.custom_id(random_button_id)
				.label("Random")
				.style(ButtonStyle::Secondary)))
	}
}

impl InteractionRender for PlaceRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		msg
		.ephemeral(true)
		.embed(|e| self.create_embed(e))
		.components(|c| self.create_components(c))
	}
}

impl FollowUpRender for PlaceRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		msg
		.ephemeral(true)
		.embed(|e| self.create_embed(e))
		.components(|c| self.create_components(c))
	}
}