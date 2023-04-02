use serenity::builder::CreateMessage;
use serenity::model::prelude::*;
use battleships_model::game_state::*;

use crate::consts::{EMBED_COLOR, ERROR_COLOR};
use crate::action::GameActionKind;

use super::MessageRender;
use super::utility::renders::SharedPrepareRender;

#[derive(Clone)]
pub struct StartRender(pub GameState);
impl MessageRender for StartRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		let user_1_id = UserId(self.0.player_1.user_id).mention();
		let user_2_id = UserId(self.0.player_2.user_id).mention();

		SharedPrepareRender(self.0, GameActionKind::Place)
		.render_message(msg)
		.add_embed(|e| e
			.description(format!("**[**{user_1_id} & {user_2_id}, get ready for battle!**]**\n**[**{user_1_id}, you prepare first.**]**"))
			.color(EMBED_COLOR))
	}
}

#[derive(Copy, Clone)]
pub enum FailStartReason {
	Bot(UserId),
	Same
}

#[derive(Clone)]
pub struct FailStartRender(pub FailStartReason);
impl MessageRender for FailStartRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		msg
		.add_embed(|e| e
			.description(match self.0 {
				FailStartReason::Bot(id) => format!("**[**{} is a bot and cannot play.**]**", id.mention()),
    			FailStartReason::Same => format!("**[**You can't play against yourself.**]**"),
			})
			.color(ERROR_COLOR))
	}
}
