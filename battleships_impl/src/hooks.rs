use serenity::model::prelude::*;
use serenity::model::application::interaction::modal::*;
use serenity::model::application::interaction::message_component::*;
use serenity::model::prelude::component::ActionRowComponent;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::*;

use battleships_model::game_state::*;

use crate::action::*;
use crate::consts::*;
use crate::render::*;

pub async fn handle_component_interaction(ctx: &Context, interaction: &MessageComponentInteraction) -> SerenityResult {
	match GameAction::from_id(&interaction.data.custom_id) {
		Ok(action) => {
			handle_component_game_action(ctx, interaction, action).await?;
		}
		Err(err) => {
			dbg!(err);
		}
	};

	Ok(())
}

pub async fn handle_modal_interaction(ctx: &Context, interaction: &ModalSubmitInteraction) -> SerenityResult {
	match GameAction::from_id(&interaction.data.custom_id) {
		Ok(action) => {
			handle_interaction_game_action(ctx, interaction, action).await?;
		}
		Err(err) => {
			dbg!(err);
		}
	};

	Ok(())
}

pub async fn start_game(ctx: &Context, channel_id: ChannelId, player_1_id: UserId, player_2_id: UserId) -> SerenityResult {
	let state = GameState::new(player_1_id.0, player_2_id.0);
	let state = StartRender(state);
	channel_id.send_message(ctx, |m| state.render_message(m)).await?;
	Ok(())
}

async fn handle_component_game_action(ctx: &Context, interaction: &MessageComponentInteraction, action: GameAction) -> SerenityResult {
	match action.kind {
		GameActionKind::StartTurn => {
			let (curr_turn, other_turn) = action.state.turns();

			if curr_turn.user_id == interaction.user.id.0 {
				// Remove the button.
				interaction.create_interaction_response(ctx, |r| r
					.interaction_response_data(|d| RemoveButtonsRender.render_interaction(d))
					.kind(InteractionResponseType::UpdateMessage)
				).await?;

				// Send the fire info.
				let state = FireRender(action.state);
				interaction.create_followup_message(ctx, |f| state.render_follow_up(f)).await?;
			} else if other_turn.user_id == interaction.user.id.0 {
				// The enemy has clicked
				interaction.create_interaction_response(ctx, |r| r
					.interaction_response_data(|d| NotYourTurnRender.render_interaction(d))
					.kind(InteractionResponseType::ChannelMessageWithSource)
				).await?;
			} else {
				// Some other random user has clicked
				interaction.create_interaction_response(ctx, |r| r
					.interaction_response_data(|d| NotInvolvedRender.render_interaction(d))
					.kind(InteractionResponseType::ChannelMessageWithSource)
				).await?;
			}

			Ok(())
		}
		GameActionKind::Fire => {
			// Let's not bother checking the user since the button is on an ephemeral message
			let state = FireModalRender(action.state);
			interaction.create_interaction_response(ctx, |f| f
				.interaction_response_data(|d| state.render_interaction(d))
				.kind(InteractionResponseType::Modal)
			).await?;

			Ok(())
		}
	}
}

async fn handle_interaction_game_action(ctx: &Context, interaction: &ModalSubmitInteraction, mut action: GameAction) -> SerenityResult {
    match action.kind {
        GameActionKind::Fire => {
			// Grab the first component. It should be the only one, so we do no more checks.
			if let Some(component) = interaction.data.components.iter().flat_map(|v| v.components.iter()).next() {
				if let ActionRowComponent::InputText(component) = component {
					// Remove the button; if needed, we'll add another
					interaction.create_interaction_response(ctx, |r| r
						.interaction_response_data(|d| RemoveButtonsRender.render_interaction(d))
						.kind(InteractionResponseType::UpdateMessage)
					).await?;

					if let Some(Coord(coord)) = Coord::from_str(&component.value) {
						let target = action.state.target_mut();
						if target.hits.get(coord) {
							// If the coordinate is already hit, tell the user that and let them take another turn
							respond_invalid_fire(ctx, interaction, action, InvalidFireReason::AlreadyHit).await?;
						} else {
							// Mark the coordinate as hit
							target.hits.set(coord);

							// Grab the info for the next turn.
							let next_turn_info =
								if let Some(ship) = target.overlap(coord) {
									if target.is_sunk(&ship) {
										NextTurnInfo::Sunk(ship.info.label)
									} else {
										NextTurnInfo::Hit
									}
								} else {
									NextTurnInfo::Miss
								};

							// Swap turns, and send a message
							action.state.swap_turn();

							let state = NextTurnRender(action.state, component.value.clone(), next_turn_info);
							interaction.channel_id.send_message(ctx, |m| state.render_message(m)).await?;
						}
					} else {
						// Invalid coordinate, report to user and let them take another turn
						respond_invalid_fire(ctx, interaction, action, InvalidFireReason::InvalidCoord).await?;
					}
				}
			}

			Ok(())
		}
        GameActionKind::StartTurn => {
			dbg!(action);
			Ok(())
		}
    }
}

async fn respond_invalid_fire(ctx: &Context, interaction: &ModalSubmitInteraction, action: GameAction, reason: InvalidFireReason) -> SerenityResult {
    let state = InvalidFireRender(action.state, reason);
    interaction.create_followup_message(ctx, |f| state.render_follow_up(f)).await?;
	Ok(())
}
