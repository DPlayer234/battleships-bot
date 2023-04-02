use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use serenity::model::prelude::*;
use serenity::model::application::interaction::Interaction;
use serenity::prelude::*;

use battleships_impl::hooks;

struct Handler {
	user_id: AtomicU64
}

impl Handler {
	fn new() -> Self {
		Handler { user_id: AtomicU64::default() }
	}
}

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _: Context, ready: Ready) {
		self.user_id.store(ready.user.id.0, Ordering::Relaxed);
		println!("{} is connected!", ready.user.name);
	}

	async fn resume(&self, _: Context, _: ResumedEvent) {
		println!("Resumed connection.");
	}

	async fn guild_create(&self, _: Context, guild: Guild) {
		println!("Joined Guild: {:?} [{}]", guild.name, guild.id);
	}

	// TODO: Handler errors returned by `hooks`
	async fn message(&self, ctx: Context, new_message: Message) {
		let user_id = UserId(self.user_id.load(Ordering::Relaxed));
		if !new_message.mentions_user_id(user_id) { return; }
		if let Some(other_player) = to_scalar(new_message.mentions.iter().filter(|&u| u.id != user_id)) {
			generic_handler(hooks::start_game(&ctx, new_message.channel_id, new_message.author.id, other_player.id).await);
		}
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		match interaction {
			Interaction::MessageComponent(ref m) => generic_handler(hooks::handle_component_interaction(&ctx, m).await),
			Interaction::ModalSubmit(ref m) => generic_handler(hooks::handle_modal_interaction(&ctx, m).await),
			_ => println!("unexpected modal interaction type: {interaction:?}")
		}
	}
}

fn to_scalar<T>(mut iter: impl Iterator<Item = T>) -> Option<T> {
	match iter.next() {
		Some(v) if iter.next().is_none() => Some(v),
		_ => None
	}
}

fn generic_handler<T, E: Error + Debug>(result: Result<T, E>) {
	if let Err(err) = result {
		dbg!(err);
	}
}

#[tokio::main]
async fn main() {
	let token = env::var("BOT_TOKEN").expect("BOT_TOKEN env var must be set.");
	let intents
		= GatewayIntents::GUILDS // Needed for a complete cache
		| GatewayIntents::GUILD_MESSAGES // Needed because this bot uses message commands
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let mut builder = Client::builder(token, intents)
		.event_handler(Handler::new())
		.await
		.expect("Error creating client.");

	if let Err(reason) = builder.start().await {
		println!("Client Error: {reason:?}");
	}
}
