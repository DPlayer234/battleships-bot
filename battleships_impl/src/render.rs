use serenity::builder::{CreateMessage, CreateEmbed, CreateInteractionResponseData, CreateInteractionResponseFollowup, CreateComponents};
use serenity::model::prelude::*;
use serenity::model::prelude::component::{ButtonStyle, InputTextStyle};
use serenity::utils::*;

use battleships_model::game_state::*;
use crate::consts::{EMBED_COLOR, ERROR_COLOR, RETRY_COLOR};
use crate::action::{GameAction, GameActionKind};

pub trait InteractionRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a>;
}

pub trait FollowUpRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a>;
}

pub trait MessageRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>;
}

pub trait EmbedRender {
	fn render_embed<'a>(self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;
}

#[derive(Clone)]
struct SharedPrepareRender(pub GameState);
impl MessageRender for SharedPrepareRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		let button_id = GameAction::new(GameActionKind::StartTurn, self.0).to_id();
		dbg!(&button_id);

		msg
		.components(|c| c
			.create_action_row(|r| r
				.create_button(|b| b
					.custom_id(button_id)
					.label("Prepare")
					.style(ButtonStyle::Primary))))
	}
}


#[derive(Clone)]
pub enum NextTurnInfo {
	Miss,
	Hit,
	Sunk(&'static str)
}

#[derive(Clone)]
pub struct NextTurnRender(pub GameState, pub String, pub NextTurnInfo);
impl MessageRender for NextTurnRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		let mut desc = MessageBuilder::new();
		desc.push_bold('[')
			.mention(&UserId(self.0.target().user_id))
			.push(" fired at ")
			.push_italic(self.1)
			.push(". ");

		match self.2 {
			NextTurnInfo::Miss => desc.push("It MISSED!"),
			NextTurnInfo::Hit => desc.push("It HIT!"),
			NextTurnInfo::Sunk(kind) => desc.push("It HIT and a ").push_italic(kind).push(" was SUNK!"),
		};

		desc.push_bold(']')
			.push('\n')
			.push_bold('[')
			.mention(&UserId(self.0.current_turn().user_id))
			.push(", it's your turn!")
			.push_bold(']');

		SharedPrepareRender(self.0)
		.render_message(msg)
		.add_embed(|e| e
			.description(desc)
			.color(EMBED_COLOR))
	}
}

#[derive(Clone)]
pub struct StartRender(pub GameState);
impl MessageRender for StartRender {
	fn render_message<'a, 'b>(self, msg: &'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
		let user_1_id = UserId(self.0.player_1.user_id).mention();
		let user_2_id = UserId(self.0.player_2.user_id).mention();

		SharedPrepareRender(self.0)
		.render_message(msg)
		.add_embed(|e| e
			.description(format!("**[**{user_1_id} & {user_2_id}, get ready for battle!**]**\n**[**{user_1_id}, it's your turn!**]**"))
			.color(EMBED_COLOR))
	}
}

struct Emotes {
	empty: &'static str,
	miss: &'static str,
	ship: &'static str,
	ship_start: &'static str,
	ship_end: &'static str,
	ship_hit: &'static str,
	ship_sunk: &'static str
}

impl Emotes {
	const ENEMY: Emotes = Emotes {
		empty: "⬛",
		miss: "🔷",
		ship: "⬛",
		ship_start: "⬛",
		ship_end: "⬛",
		ship_hit: "🟥",
		ship_sunk: "❌"
	};

	const OWN: Emotes = Emotes {
		empty: "🟦",
		miss: "🔷",
		ship: "⬜",
		ship_start: "◻️",
		ship_end: "◻️",
		ship_hit: "🟥",
		ship_sunk: "❌"
	};
}

#[derive(Clone)]
pub struct FireRender(pub GameState);
impl FollowUpRender for FireRender {
	fn render_follow_up<'a, 'b>(self, msg: &'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
		const BUF_SIZE: usize = GRID_SIZE as usize;

		bitflags::bitflags! {
			#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
			struct Field: u8 {
				const NONE = 0;
				const HIT = 1;
				const SUNK = 2;
				const SHIP = 4;
				const SHIP_START = 8;
				const SHIP_END = 16;
			}
		}
		
		type Buffer = [[Field; BUF_SIZE]; BUF_SIZE];

		fn get_field(buf: &mut Buffer, pos: Vec2) -> Option<&mut Field> {
			buf.get_mut(pos.y as usize).and_then(|r| r.get_mut(pos.x as usize))
		}

		fn sub_render(buf: &mut Buffer, player: &PlayerState) {
			// First, mark field hits from the HitMatrix
			for x in 0..GRID_SIZE {
				for y in 0..GRID_SIZE {
					if player.hits.get(Vec2::new(x, y)) {
						buf[y as usize][x as usize] |= Field::HIT;
					}
				}
			}

			for ship in player.ships() {
				let pos = ship.state.position();
				let rot = ship.state.rotation();
				let len = ship.info.len;

				// Track whether the ship was sunk
				let sunk = player.is_sunk(&ship);

				// Mark parts as ship with appropriate flags.
				for i in 0..len {
					let mut t_pos = pos;
					if rot == Rotation::HORI { t_pos.x += i; } else { t_pos.y += i; }
					if let Some(field) = get_field(buf, t_pos) {
						let mut flags =
							if i == 0 { Field::SHIP | Field::SHIP_START }
							else if i >= len - 1 { Field::SHIP | Field::SHIP_END }
							else { Field::SHIP };

						if sunk { flags |= Field::SUNK; }

						*field |= flags;
					}
				}
			}
		}

		fn render_grid(title: &str, buf: &Buffer, emotes: &Emotes) -> String {
			const HEADER: &str = "\n🌊\u{feff}🇦\u{feff}🇧\u{feff}🇨\u{feff}🇩\u{feff}🇪\u{feff}🇫\u{feff}🇬\u{feff}🇭\u{feff}🇮\u{feff}🇯\n";
			const NUMS: &[&str] = &[
				"1\u{fe0f}\u{20e3}", "2\u{fe0f}\u{20e3}", "3\u{fe0f}\u{20e3}",
				"4\u{fe0f}\u{20e3}", "5\u{fe0f}\u{20e3}", "6\u{fe0f}\u{20e3}",
				"7\u{fe0f}\u{20e3}", "8\u{fe0f}\u{20e3}", "9\u{fe0f}\u{20e3}",
				"🔟"
			];

			let mut res = String::new();
			res.push_str(title);
			res.push_str(HEADER);

			for (index, row) in buf.iter().enumerate() {
				if let Some(&num) = NUMS.get(index) {
					res.push_str(num);
				}

				for &field in row {
					res.push_str(
						if field == Field::NONE { emotes.empty }
						else if field == Field::HIT { emotes.miss }
						else if field.contains(Field::SUNK) { emotes.ship_sunk }
						else if field.contains(Field::SHIP | Field::HIT) { emotes.ship_hit }
						else if field.contains(Field::SHIP_START) { emotes.ship_start }
						else if field.contains(Field::SHIP_END) { emotes.ship_end }
						else { emotes.ship }
					);
				}

				res.push('\n');
			}

			res
		}

		let (curr_turn, other_turn) = self.0.turns();

		let mut buffer = Buffer::default();
		sub_render(&mut buffer, &other_turn);
		
		msg
		.embed(|e| e
			.description(
				render_grid(
					&format!("**[Enemy]** {}", UserId(other_turn.user_id).mention()),
					&buffer,
					&Emotes::ENEMY))
			.colour(EMBED_COLOR));

		buffer = Buffer::default();
		sub_render(&mut buffer, &curr_turn);

		msg
		.embed(|e| e
			.description(
				render_grid(
					&format!("**[Own]** {}", UserId(curr_turn.user_id).mention()),
					&buffer,
					&Emotes::OWN))
			.colour(EMBED_COLOR));

		let button_id = GameAction::new(GameActionKind::Fire, self.0).to_id();
		dbg!(&button_id);

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
		dbg!(&button_id);

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

const FIRE_TEXT_ID: &str = "fire";

#[derive(Clone)]
pub struct FireModalRender(pub GameState);
impl InteractionRender for FireModalRender {
	fn render_interaction<'a, 'b>(self, msg: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
		let custom_id = GameAction::new(GameActionKind::Fire, self.0).to_id();
		dbg!(&custom_id);

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
