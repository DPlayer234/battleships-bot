use serenity::builder::{CreateInteractionResponseFollowup};
use serenity::model::prelude::*;
use serenity::utils::*;
use battleships_model::game_state::*;

use crate::consts::EMBED_COLOR;
use crate::action::{GameActionKind, Coord};

use super::FollowUpRender;
use super::utility::renders::SharedPrepareRender;

#[derive(Clone)]
pub struct FirstTurnRender(pub GameState);
impl FollowUpRender for FirstTurnRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		let mut desc = MessageBuilder::new();

		desc.push_bold('[')
			.mention(&UserId(self.0.current().user_id))
			.push(", it's your turn!")
			.push_bold(']');
	
		SharedPrepareRender(self.0, GameActionKind::StartTurn)
		.render_follow_up(msg)
		.ephemeral(false)
		.embed(|e| e
			.description(desc)
			.color(EMBED_COLOR))
	}
}

#[derive(Clone)]
pub enum NextTurnInfo {
	Miss,
	Hit,
	Sunk { kind: &'static str, loss: bool }
}

#[derive(Clone)]
pub struct NextTurnRender { pub state: GameState, pub tile: Coord, pub info: NextTurnInfo }
impl FollowUpRender for NextTurnRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		// Current turn is the one that was shot *AT*
		// Target turn was the shooter
		// So if it's Sunk with loss=true, the current player lost.

		let mut desc = MessageBuilder::new();
		desc.push_bold('[')
			.mention(&UserId(self.state.target().user_id))
			.push(" fired at ")
			.push_italic(self.tile.to_string())
			.push(". ");

		match self.info {
			NextTurnInfo::Miss => desc.push("It MISSED!"),
			NextTurnInfo::Hit => desc.push("It HIT!"),
			NextTurnInfo::Sunk { kind, .. } => desc.push("It HIT and a ").push_italic(kind).push(" was SUNK!")
		};

		desc.push_bold(']')
			.push('\n');

		if matches!(self.info, NextTurnInfo::Sunk { loss: true, .. }) {
			let (loser, winner) = self.state.turns();

			desc.push_bold('[')
				.mention(&UserId(loser.user_id))
				.push(" lost all their ships! ")
				.mention(&UserId(winner.user_id))
				.push(" wins!")
				.push_bold(']');
		} else {
			desc.push_bold('[')
				.mention(&UserId(self.state.current().user_id))
				.push(", it's your turn!")
				.push_bold(']');
	
			SharedPrepareRender(self.state, GameActionKind::StartTurn)
			.render_follow_up(msg);
		}

		msg
		.ephemeral(false)
		.embed(|e| e
			.description(desc)
			.color(EMBED_COLOR))
	}
}
