use serenity::builder::{CreateInteractionResponseData, CreateInteractionResponseFollowup};
use serenity::model::prelude::*;
use serenity::model::prelude::component::{ButtonStyle, InputTextStyle};

use battleships_model::game_state::*;
use crate::consts::{EMBED_COLOR, RETRY_COLOR};
use crate::action::{GameAction, GameActionKind};

use super::{FollowUpRender, InteractionRender};
use super::utility::render_target::*;

#[derive(Clone)]
pub struct FireRender(pub GameState);
impl FollowUpRender for FireRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		let (curr_turn, other_turn) = self.0.turns();

		let mut buffer = RenderTarget::new();
		buffer.set_all_fields(&other_turn);
		
		msg
		.embed(|e| e
			.description(
				buffer.render_grid(
					&format!("**[Enemy]** {}", UserId(other_turn.user_id).mention()),
					&Emotes::ENEMY))
			.colour(EMBED_COLOR));

		buffer = RenderTarget::new();
		buffer.set_all_fields(&curr_turn);

		msg
		.embed(|e| e
			.description(
				buffer.render_grid(
					&format!("**[Own]** {}", UserId(curr_turn.user_id).mention()),
					&Emotes::OWN))
			.colour(EMBED_COLOR));

		let button_id = GameAction::new(GameActionKind::Fire, self.0).to_id();

		msg
		.ephemeral(true)
		.components(|c| c
			.create_action_row(|r| r
				.create_button(|b| b
					.custom_id(button_id)
					.label("Fire")
					.style(ButtonStyle::Primary))))
	}
}

#[derive(Copy, Clone)]
pub enum InvalidFireReason {
	InvalidCoord,
	AlreadyHit
}

#[derive(Clone)]
pub struct InvalidFireRender(pub GameState, pub InvalidFireReason);
impl FollowUpRender for InvalidFireRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		let button_id = GameAction::new(GameActionKind::Fire, self.0).to_id();

		let reason = match self.1 {
			InvalidFireReason::InvalidCoord => "**[**That coordinate is invalid.**]**",
			InvalidFireReason::AlreadyHit => "**[**You already fired at that coordinate.**]**",
		};

		msg
		.ephemeral(true)
		.embed(|e| e
			.description(reason)
			.colour(RETRY_COLOR))
		.components(|c| c
			.create_action_row(|r| r
				.create_button(|b| b
					.custom_id(button_id)
					.label("Fire")
					.style(ButtonStyle::Primary))))
	}
}

const FIRE_TEXT_ID: &str = "fire";

#[derive(Clone)]
pub struct ChooseFireRender(pub GameState);
impl InteractionRender for ChooseFireRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		let custom_id = GameAction::new(GameActionKind::Fire, self.0).to_id();

		msg
		.custom_id(custom_id)
		.title("Enter Target")
		.components(|c| c
			.create_action_row(|r| r
				.create_input_text(|i| i
					.custom_id(FIRE_TEXT_ID)
					.label("Tile")
					.placeholder("f.e. B4")
					.min_length(2)
					.max_length(3)
					.style(InputTextStyle::Short)
					.required(true))))
	}
}