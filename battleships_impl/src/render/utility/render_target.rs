use battleships_model::game_state::{PlayerState, Vec2, GRID_SIZE};

pub struct Emotes {
	empty: &'static str,
	miss: &'static str,
	ship: &'static str,
	ship_start: &'static str,
	ship_end: &'static str,
	ship_hit: &'static str,
	ship_sunk: &'static str
}

impl Emotes {
	pub const ENEMY: Emotes = Emotes {
		empty: "⬛",
		miss: "🔷",
		ship: "⬛",
		ship_start: "⬛",
		ship_end: "⬛",
		ship_hit: "🟥",
		ship_sunk: "❌"
	};

	pub const OWN: Emotes = Emotes {
		empty: "🟦",
		miss: "🔷",
		ship: "⬜",
		ship_start: "◻️",
		ship_end: "◻️",
		ship_hit: "🟥",
		ship_sunk: "❌"
	};
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
	struct RenderFlags: u8 {
		const NONE = 0;
		const HIT = 1;
		const SUNK = 2;
		const SHIP = 4;
		const SHIP_START = 8;
		const SHIP_END = 16;
	}
}

const RENDER_SIZE: usize = GRID_SIZE as usize;

#[derive(Debug, Clone)]
pub struct RenderTarget {
	buffer: [[RenderFlags; RENDER_SIZE]; RENDER_SIZE]
}

impl RenderTarget {
	pub fn new() -> Self {
		RenderTarget { buffer: Default::default() }
	}

	fn get_field_mut(&mut self, pos: Vec2) -> Option<&mut RenderFlags> {
		self.buffer.get_mut(pos.y as usize).and_then(|r| r.get_mut(pos.x as usize))
	}

	pub fn set_all_fields(&mut self, player: &PlayerState) {
		self.set_hits(player);
		self.set_ships(player);
	}

	pub fn set_hits(&mut self, player: &PlayerState) {
		for x in 0..GRID_SIZE {
			for y in 0..GRID_SIZE {
				if player.hits.get(Vec2::new(x, y)) {
					self.buffer[y as usize][x as usize] |= RenderFlags::HIT;
				}
			}
		}
	}

	pub fn set_ships(&mut self, player: &PlayerState) {
		for ship in player.ships() {
			// Track whether the ship was sunk
			let sunk = player.is_sunk(&ship);
			let len = usize::from(ship.info.len);

			// Mark parts as ship with appropriate flags.
			for (i, pos) in ship.tiles().enumerate() {
				if let Some(field) = self.get_field_mut(pos) {
					let mut flags =
						if i == 0 { RenderFlags::SHIP | RenderFlags::SHIP_START }
						else if i >= len - 1 { RenderFlags::SHIP | RenderFlags::SHIP_END }
						else { RenderFlags::SHIP };
					if sunk { flags |= RenderFlags::SUNK; }

					*field |= flags;
				}
			}
		}
	}

	pub fn render_grid(&self, title: &str, emotes: &Emotes) -> String {
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

		for (index, row) in self.buffer.iter().enumerate() {
			if let Some(&num) = NUMS.get(index) {
				res.push_str(num);
			}

			for &field in row {
				res.push_str(
					if field == RenderFlags::NONE { emotes.empty }
					else if field == RenderFlags::HIT { emotes.miss }
					else if field.contains(RenderFlags::SUNK) { emotes.ship_sunk }
					else if field.contains(RenderFlags::SHIP | RenderFlags::HIT) { emotes.ship_hit }
					else if field.contains(RenderFlags::SHIP_START) { emotes.ship_start }
					else if field.contains(RenderFlags::SHIP_END) { emotes.ship_end }
					else { emotes.ship }
				);
			}

			res.push('\n');
		}

		res
	}
}
