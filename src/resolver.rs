use crate::blocklist::Blocklist;
use crate::cache::Cache;
use crate::dns::Message as DNSMessage;
use crate::requests::Request;
use crate::requests::Respondable;
use crate::settings::{ResolverSettings, UpstreamSettings};
use crate::upstreams::Upstreams;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Resolver {
    blocklist: Blocklist,
    queue: Arc<Mutex<VecDeque<(Request)>>>,
    cache: Cache,
    upstreams: Upstreams,
}

impl Resolver {
    pub fn new(
        queue: Arc<Mutex<VecDeque<(Request)>>>,
        resolver_settings: &ResolverSettings,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            blocklist: Blocklist::new(),
            queue,
            cache: Cache::new(&resolver_settings.cache),
            upstreams: Upstreams::new(&resolver_settings.upstreams),
        })
    }

    pub fn start(self: Arc<Self>) -> Result<(), io::Error> {
        let queue_clone = self.queue.clone();

        thread::spawn(move || {
            loop {
                let mut queue = queue_clone.lock().unwrap();
                if let Some(request) = queue.pop_front() {
                    drop(queue); // Drop the lock as soon as possible
                    self.process_message(request);
                } else {
                    drop(queue);
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        Ok(())
    }

    fn process_message(&self, request: Request) {
        let request_message = request.message;
        let request_domain = request_message.qname_to_string();

        if self.blocklist.contains(&request_domain) {
            println!("Domain {} is blocked.", request_domain);
            return;
        }

        if let Some(cached_response) = self.cache.query(&request_domain) {
            println!("Cache hit for domain: {}", request_domain);
            // Send cached_response back to the client
            return;
        }

        if let Some(response) = self.upstreams.query(&request_domain) {
            println!("Response from upstream for domain: {}", request_domain);
            // Send response back to the client
            return;
        }

        println!("Domain {} not found.", request_domain);
        let response = DNSMessage::new_not_found_response(&request_message);
        let response_data = response.serialize();
        request
            .connection_info
            .send_response(&response_data)
            .unwrap();
    }
}
