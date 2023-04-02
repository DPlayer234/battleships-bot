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
			handle_component_game_action(ctx, interaction, action).await
		}
		Err(err) => {
			dbg!(err);
			Ok(())
		}
	}
}

pub async fn handle_modal_interaction(ctx: &Context, interaction: &ModalSubmitInteraction) -> SerenityResult {
	match GameAction::from_id(&interaction.data.custom_id) {
		Ok(action) => {
			handle_interaction_game_action(ctx, interaction, action).await
		}
		Err(err) => {
			dbg!(err);
			Ok(())
		}
	}
}

pub async fn start_game(ctx: &Context, channel_id: ChannelId, player_1: &User, player_2: &User) -> SerenityResult {
	match check_players(player_1, player_2) {
		Ok(_) => {
			let state = GameState::new(player_1.id.0, player_2.id.0);
			let state = StartRender(state);
			channel_id.send_message(ctx, |m| state.render_message(m)).await?;
		}
		Err(reason) => {
			let state = FailStartRender(reason);
			channel_id.send_message(ctx, |m| state.render_message(m)).await?;
		}
	}

	Ok(())
}

async fn handle_component_game_action(ctx: &Context, interaction: &MessageComponentInteraction, mut action: GameAction) -> SerenityResult {
	if action.state.current().user_id != interaction.user.id.0 {
		return if action.state.target().user_id == interaction.user.id.0 {
			// The enemy has clicked
			render_interaction_response(ctx, interaction, InteractionResponseType::ChannelMessageWithSource, NotYourTurnRender).await
		} else {
			// Some other random user has clicked
			render_interaction_response(ctx, interaction, InteractionResponseType::ChannelMessageWithSource, NotInvolvedRender).await
		};
	}

	match action.kind {
		GameActionKind::StartTurn => {
			render_follow_up_and_delete_buttons(ctx, interaction, FireRender(action.state)).await
		}
		GameActionKind::Fire => {
			render_interaction_response(ctx, interaction, InteractionResponseType::Modal, ChooseFireRender(action.state)).await
		}
		GameActionKind::Place => {
			render_follow_up_and_delete_buttons(ctx, interaction, PlaceRender(action.state)).await
		}
		GameActionKind::RandomizePlace => {
			action.state.current_mut().randomize_ships();
			render_interaction_response(ctx, interaction, InteractionResponseType::UpdateMessage, PlaceRender(action.state)).await
		}
		GameActionKind::ConfirmPlace => {
			if action.state.turn_num() == 1 {
				// If Player 1 chose, we also ask Player 2 to prepare
				action.state.swap_turn();
				render_follow_up_and_delete_buttons(ctx, interaction, NextPlaceRender(action.state)).await
			} else {
				// If Player 2 confirms, that means both players are ready
				action.state.swap_turn();
				render_follow_up_and_delete_buttons(ctx, interaction, FirstTurnRender(action.state)).await
			}
		}

		#[allow(unreachable_patterns)]
		_ => {
			dbg!(action);
			Ok(())
		}
	}
}

async fn render_interaction_response(ctx: &Context, interaction: &MessageComponentInteraction, kind: InteractionResponseType, state: impl InteractionRender) -> SerenityResult {
	interaction.create_interaction_response(ctx, |f| f
		.interaction_response_data(|d| state.render_interaction(d))
		.kind(kind)
	).await
}

async fn render_follow_up_and_delete_buttons(ctx: &Context, interaction: &MessageComponentInteraction, state: impl FollowUpRender) -> SerenityResult {
	// Remove the button.
	interaction.create_interaction_response(ctx, |r| r
		.interaction_response_data(|d| RemoveButtonsRender.render_interaction(d))
		.kind(InteractionResponseType::UpdateMessage)
	).await?;

	// Send the follow up
	interaction.create_followup_message(ctx, |f| state.render_follow_up(f)).await?;
	Ok(())
}

async fn handle_interaction_game_action(ctx: &Context, interaction: &ModalSubmitInteraction, mut action: GameAction) -> SerenityResult {
    match action.kind {
        GameActionKind::Fire => {
			// Grab the first component. It should be the only one, so we do no more checks.
			let component = interaction.data.components.iter().flat_map(|v| v.components.iter()).next();
			let Some(ActionRowComponent::InputText(component)) = component else { return Ok(()); };
			
			// Remove the button; if needed, we'll add another
			interaction.create_interaction_response(ctx, |r| r
				.interaction_response_data(|d| RemoveButtonsRender.render_interaction(d))
				.kind(InteractionResponseType::UpdateMessage)
			).await?;

			let Some(Coord(coord)) = Coord::from_str(&component.value) else {
				// Invalid coordinate, report to user and let them take another turn
				return respond_invalid_fire(ctx, interaction, action, InvalidFireReason::InvalidCoord).await;
			};

			let target = action.state.target_mut();
			if target.hits.get(coord) {
				// If the coordinate is already hit, tell the user that and let them take another turn
				respond_invalid_fire(ctx, interaction, action, InvalidFireReason::AlreadyHit).await?;
			} else {
				// Mark the coordinate as hit
				target.hits.set(coord);

				// Grab the info for the next turn.
				let next_turn_info = match target.overlap(coord) {
					Some(ref s) if target.is_sunk(s) => NextTurnInfo::Sunk { kind: s.info.label, loss: target.are_all_ships_sunk() },
					Some(_) => NextTurnInfo::Hit,
					None => NextTurnInfo::Miss
				};

				// Swap turns, and send a message
				action.state.swap_turn();

				let state = NextTurnRender {
					state: action.state,
					tile: Coord(coord),
					info: next_turn_info
				};

				interaction.create_followup_message(ctx, |m| state.render_follow_up(m)).await?;
			}

			Ok(())
		}
		
		#[allow(unreachable_patterns)]
        _ => {
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

fn check_players(player_1: &User, player_2: &User) -> std::result::Result<(), FailStartReason> {
	if player_1.id == player_2.id {
		Err(FailStartReason::Same)
	} else {
		check_player(player_1).and_then(|_| check_player(player_2))
	}
}

fn check_player(user: &User) -> std::result::Result<(), FailStartReason> {
	if user.bot {
		Err(FailStartReason::Bot(user.id))
	} else {
		Ok(())
	}
}
