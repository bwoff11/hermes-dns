use std::io::{self, Write};
use std::sync::Arc;

mod blocklist;
mod cache;
mod dns;
mod listeners;
mod resolver;
mod requests;
mod settings;
mod upstreams;

#[tokio::main]
async fn main() {
    let settings = settings::Settings::load().expect("Failed to load settings");
    let queue = listeners::Listeners::new(&settings.listeners)
        .await
        .expect("Failed to create new listeners")
        .listen()
        .await
        .expect("Failed to listen on listeners");

    let resolver =
        resolver::Resolver::new(queue, &settings.resolver).expect("Failed to create resolver");

        resolver.start().await;

    println!("Press Enter to exit...");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut exit_command = String::new();
    io::stdin().read_line(&mut exit_command).expect("Failed to read line");
    println!("Exiting...");
}

