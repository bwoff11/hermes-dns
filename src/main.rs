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

fn main() {
    let settings = settings::Settings::load().expect("Failed to load settings");
    let queue = listeners::Listeners::new(&settings.listeners).start();

    let resolver =
        resolver::Resolver::new(queue, &settings.resolver).expect("Failed to create resolver");
    let arc_resolver = Arc::new(resolver);

    arc_resolver.start().expect("Failed to start resolver");

    println!("Press Enter to exit...");

    // Flush stdout to ensure the message is displayed before blocking on input
    io::stdout().flush().expect("Failed to flush stdout");

    // Wait for the user to press Enter
    let mut exit_command = String::new();
    io::stdin()
        .read_line(&mut exit_command)
        .expect("Failed to read line");

    println!("Exiting...");
}
