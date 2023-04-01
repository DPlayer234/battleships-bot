use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};

use battleships_model::encode::Encode;
use battleships_model::game_state::{GameState, Vec2, GRID_SIZE};

use crate::consts::CUSTOM_ID_PREFIX;

#[derive(Debug, Clone)]
pub struct GameAction {
	pub kind: GameActionKind,
	pub state: GameState
}

#[derive(Debug, Copy, Clone)]
pub enum GameActionKind {
	StartTurn,
	Fire
}

#[derive(Debug, Copy, Clone)]
pub enum GameActionParseError {
	NotBattleshipId,
	UnknownAction,
	InvalidData,
	NoData
}

#[derive(Debug, Copy, Clone)]
pub struct Coord(pub Vec2);

impl GameAction {
	pub fn new(kind: GameActionKind, state: GameState) -> Self {
		Self { kind, state }
	}

	pub fn to_id(self) -> String {
		let mut id = CUSTOM_ID_PREFIX.to_owned();
		id.push(self.kind.to_char());
		STANDARD_NO_PAD.encode_string(self.state.encode(), &mut id);
		id
	}

	pub fn from_id(mut id: &str) -> Result<Self, GameActionParseError> {
		if !id.starts_with(CUSTOM_ID_PREFIX) {
			return Err(GameActionParseError::NotBattleshipId);
		}
	
		// Let's just assume it's ASCII
		// We already checked that it starts with this constant
		id = &id[CUSTOM_ID_PREFIX.len()..];

		let kind = id.chars().nth(0).ok_or(GameActionParseError::NoData)?;
		let kind = GameActionKind::from_char(kind)?;
		let state = parse_game_state(&id[1..])?;

		Ok(GameAction { kind, state })
	}
}

impl GameActionKind {
	pub fn to_char(self) -> char {
		match self {
			GameActionKind::StartTurn => 'T',
			GameActionKind::Fire => 'F'
		}
	}

	pub fn from_char(c: char) -> Result<GameActionKind, GameActionParseError>{
		match c {
			'T' => Ok(GameActionKind::StartTurn),
			'F' => Ok(GameActionKind::Fire),
			_ => Err(GameActionParseError::UnknownAction)
		}
	}
}

impl Coord {
	pub fn from_str(t: &str) -> Option<Coord> {
		let mut chrs = t.chars();
		if let Some(column) = chrs.next() {
			return if !column.is_ascii() { None }
			else { Self::from_pair(column, &t[1..]) }
		}

		None
	}

	fn from_pair(column: char, row: &str) -> Option<Coord> {
		// J = Column 10
		const UP_A: u32 = 'A' as u32;
		const UP_J: u32 = 'J' as u32;

		const LOW_A: u32 = 'a' as u32;
		const LOW_J: u32 = 'J' as u32;

		let column: u32 = column.into();
		let column =
			if column >= UP_A && column <= UP_J { column - UP_A }
			else if column >= LOW_A && column <= LOW_J { column - LOW_A }
			else { return None } as u8;

		if let Ok(row) = row.parse::<u8>() {
			if row >= 1 && row <= GRID_SIZE {
				return Some(Coord(Vec2::new(column, row - 1)));
			}
		}
		
		None
	}
}

fn parse_game_state(id: &str) -> Result<GameState, GameActionParseError> {
	if let Ok(raw) = STANDARD_NO_PAD.decode(id) {
		// This will implicitly do a length check
		// So if the vector is the wrong size, it will fail
		if let Some(state) = GameState::try_decode(&raw) {
			return Ok(state);
		}
	}

	Err(GameActionParseError::InvalidData)
}
