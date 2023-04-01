use std::fmt::Debug;
use std::mem::size_of;

use rand::distributions::{Uniform, Standard};
use rand::{thread_rng, Rng};

use crate::encode::Encode;

pub const GRID_SIZE: u8 = 10;

#[derive(Debug, Clone)]
pub struct GameState {
	pub player_1: PlayerState,
	pub player_2: PlayerState,
	turn: Turn
}

#[derive(Debug, Clone)]
pub struct PlayerState {
	pub user_id: u64,
	pub hits: HitMatrix,
	ships: [ShipState; ShipInfo::COUNT],
}

#[derive(Debug, Clone)]
pub struct HitMatrix(u128);

#[derive(Debug, Clone)]
pub struct Ship {
	pub info: &'static ShipInfo,
	pub state: ShipState
}

#[derive(Clone, Copy)]
pub struct ShipState(u8);

#[derive(Debug, Clone)]
pub struct ShipInfo {
	pub label: &'static str,
	pub index: usize,
	pub len: u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
	pub x: u8,
	pub y: u8
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rotation(u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Turn(pub u8);

impl Vec2 {
	pub const fn new(x: u8, y: u8) -> Self {
		Vec2 { x, y }
	}
}

impl GameState {
	pub fn new(player_1_id: u64, player_2_id: u64) -> Self {
		GameState {
			player_1: PlayerState::new(player_1_id),
			player_2: PlayerState::new(player_2_id),
			turn: Turn(1)
		}
	}

	pub fn current_turn(&self) -> &PlayerState {
		match self.turn.0 {
			1 => &self.player_1,
			2 => &self.player_2,
			_ => panic!("invalid turn state")
		}
	}

	pub fn target(&self) -> &PlayerState {
		// This returns the *inactive* turn
		match self.turn.0 {
			1 => &self.player_2,
			2 => &self.player_1,
			_ => panic!("invalid turn state")
		}
	}

	pub fn target_mut(&mut self) -> &mut PlayerState {
		// This returns the *inactive* turn
		match self.turn.0 {
			1 => &mut self.player_2,
			2 => &mut self.player_1,
			_ => panic!("invalid turn state")
		}
	}

	pub fn turns(&self) -> (&PlayerState, &PlayerState) {
		match self.turn.0 {
			1 => (&self.player_1, &self.player_2),
			2 => (&self.player_2, &self.player_1),
			_ => panic!("invalid turn state")
		}
	}

	pub fn swap_turn(&mut self) {
		self.turn.0 = match self.turn.0 {
			1 => 2,
			2 => 1,
			_ => panic!("invalid turn state")
		};
	}
}

// This is just cursed.
fn has_overlaps(ships: &[ShipState; ShipInfo::COUNT], index: usize) -> bool {
	let ship = Ship { info: &ShipInfo::ALL[index], state: ships[index] };
	let (b_l, b_r) = ship.bounds();

	ships.iter()
		.zip(ShipInfo::ALL)
		.take(index)
		.map(|t| {
			// Map each state into its bounds
			let (&state, info) = t;
			Ship { info, state }.bounds()
		})
		.any(move |(a_l, a_r)| {
			// Check if any of the positions mapped above
			// are in range of the current ship
			b_l.x <= a_r.x && a_l.x <= b_r.x && // Overlap on X
			b_l.y <= a_r.y && a_l.y <= b_r.y // Overlap on Y
		})
}

impl PlayerState {
	pub fn new(user_id: u64) -> Self {
		let mut rng = thread_rng();
		let mut ships = [ShipState(0); ShipInfo::COUNT];

		for i in 0..ShipInfo::COUNT {
			let info = ShipInfo::ALL[i];
			loop {
				let short = rng.sample(Uniform::new(0, GRID_SIZE - info.len));
				let full = rng.sample(Uniform::new(0, GRID_SIZE));
				let state =
					if rng.sample(Standard) {
						ShipState::new(Vec2::new(short, full), Rotation::HORI)
					} else {
						ShipState::new(Vec2::new(full, short), Rotation::VERT)
					};

				ships[i] = state;
				if !has_overlaps(&ships, i) { break; }
			}
		}

		Self {
			user_id,
			ships,
			hits: Default::default()
		}
	}

	pub fn ships(&self) -> [Ship; ShipInfo::COUNT] {
		[
			Ship { info: &ShipInfo::CARRIER, state: self.ships[0] },
			Ship { info: &ShipInfo::BATTLESHIP, state: self.ships[1] },
			Ship { info: &ShipInfo::CRUISER, state: self.ships[2] },
			Ship { info: &ShipInfo::SUBMARINE, state: self.ships[3] },
			Ship { info: &ShipInfo::DESTROYER, state: self.ships[4] },
		]
	}

	pub fn overlap(&self, pos: Vec2) -> Option<Ship> {
		self.ships().iter().find(|&s| {
			let (l, r) = s.bounds();
			pos.x >= l.x && pos.y >= l.y &&
			pos.x <= r.x && pos.y <= r.y
		}).cloned()
	}

	pub fn is_sunk(&self, ship: &Ship) -> bool {
		let mut res = true;

		for tile in ship.tiles() {
			if tile.x > GRID_SIZE || tile.y > GRID_SIZE { break; }
			res &= self.hits.get(tile);
		}

		res
	}
}

impl HitMatrix {
	pub fn new() -> Self {
		HitMatrix(0)
	}

	pub fn get(&self, slot: Vec2) -> bool {
		(self.0 & Self::get_mask(slot)) != 0
	}

	pub fn set(&mut self, slot: Vec2) {
		self.0 |= Self::get_mask(slot);
	}

	pub fn unset(&mut self, slot: Vec2) {
		self.0 &= !Self::get_mask(slot);
	}

	fn get_mask(slot: Vec2) -> u128 {
		1u128 << (slot.x + slot.y * GRID_SIZE)
	}
}

impl Default for HitMatrix {
	fn default() -> Self {
		Self::new()
	}
}

impl Ship {
	pub fn bounds(&self) -> (Vec2, Vec2) {
		let len = self.info.len;
		let rot = self.state.rotation();
		let tl_pos = self.state.position();
		let mut br_pos = tl_pos;
		if rot == Rotation::HORI { br_pos.x += len; } else { br_pos.y += len; }

		(tl_pos, br_pos)
	}

	pub fn tiles(&self) -> impl Iterator<Item = Vec2> {
		let len = self.info.len;
		let rot = self.state.rotation();
		let pos = self.state.position();

		(0..len).map(move |i| {
			let mut t_pos = pos;
			if rot == Rotation::HORI { t_pos.x += i; } else { t_pos.y += i; }
			t_pos
		})
	}
}

impl ShipState {
	pub fn new(pos: Vec2, rot: Rotation) -> Self {
		const LEN: u8 = GRID_SIZE - 1;

		assert!(pos.x < GRID_SIZE);
		assert!(pos.y < GRID_SIZE);

		if rot == Rotation::HORI {
			assert!(pos.x < LEN);
			Self(pos.x * LEN + pos.y)
		} else {
			assert!(pos.y < LEN);
			Self((pos.x + pos.y * LEN) | (1 << 7))
		}
	}

	pub fn position(self) -> Vec2 {
		const LEN: u8 = GRID_SIZE - 1;
		let val = self.0 & 0b0111_1111;
		let x = val / LEN;
		let y = val % LEN;
		if self.rotation() == Rotation::HORI {
			Vec2::new(x, y)
		} else {
			Vec2::new(y, x)
		}
	}

	pub fn rotation(self) -> Rotation {
		Rotation(self.0 >> 7)
	}
}

impl Debug for ShipState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ShipState")
			.field("position", &self.position())
			.field("rotation", &self.rotation())
			.finish()
	}
}

impl ShipInfo {
	pub const COUNT: usize = 5;

	const CARRIER: ShipInfo = ShipInfo::new("Carrier", 0, 5);
	const BATTLESHIP: ShipInfo = ShipInfo::new("Battleship", 1, 4);
	const CRUISER: ShipInfo = ShipInfo::new("Cruiser", 2, 3);
	const SUBMARINE: ShipInfo = ShipInfo::new("Submarine", 3, 3);
	const DESTROYER: ShipInfo = ShipInfo::new("Destroyer", 4, 2);

	pub const ALL: [&ShipInfo; ShipInfo::COUNT] = [
		&ShipInfo::CARRIER,
		&ShipInfo::BATTLESHIP,
		&ShipInfo::CRUISER,
		&ShipInfo::SUBMARINE,
		&ShipInfo::DESTROYER,
	];

	const fn new(label: &'static str, index: usize, len: u8) -> Self {
		Self { label, index, len }
	}
}

impl Rotation {
	pub const HORI: Self = Self(0);
	pub const VERT: Self = Self(1);
}

impl Encode for GameState {
	type Out = Vec<u8>;
	type In = [u8];

	fn encode(&self) -> Self::Out {
		let mut res = Vec::with_capacity(59);

		res.extend(self.player_1.encode());
		res.extend(self.player_2.encode());
		res.push(self.turn.0);

		debug_assert_eq!(res.len(), 59);

		res
	}

	fn try_decode(data: &Self::In) -> Option<Self> {
		if data.len() != 59 { return None; }

		let player_1 = PlayerState::try_decode(&data[0..29]).unwrap();
		let player_2 = PlayerState::try_decode(&data[29..58]).unwrap();
		let turn = Turn(data[58]);

		Some(Self {
			player_1,
			player_2,
			turn
		})
	}
}

impl Encode for PlayerState {
	type Out = Vec<u8>;
	type In = [u8];

	fn encode(&self) -> Self::Out {
		let mut res = Vec::with_capacity(29);

		res.extend(self.user_id.to_be_bytes());
		res.extend(self.hits.0.to_be_bytes());
		res.extend(self.ships.map(|s| s.0));

		debug_assert_eq!(res.len(), 29);

		res
	}

	fn try_decode(data: &Self::In) -> Option<Self> {
		if data.len() != 29 { return None; }

		const U64_SIZE: usize = size_of::<u64>();
		const U128_SIZE: usize = size_of::<u128>();

		const HITS_START: usize = U64_SIZE;
		const SHIPS_START: usize = HITS_START + U128_SIZE;

		let user_id = u64::from_be_bytes(data[..HITS_START].try_into().unwrap());
		let hits = HitMatrix(u128::from_be_bytes(data[HITS_START..SHIPS_START].try_into().unwrap()));
		let ships: [u8; ShipInfo::COUNT] = data[SHIPS_START..].try_into().unwrap();
		let ships = ships.map(|b| ShipState(b));

		Some(Self {
			user_id,
			hits,
			ships
		})
	}
}
