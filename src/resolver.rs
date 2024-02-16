use crate::blocklist::Blocklist;
use crate::cache::Cache;
use crate::dns::Message as DNSMessage;
use crate::requests::Request;
use crate::settings::{ResolverSettings, UpstreamSettings};
use crate::upstreams::Upstreams;
use tokio::sync::mpsc::Receiver;
use std::io;
use std::sync::Arc;

pub struct Resolver {
    blocklist: Blocklist,
    queue_receiver: Receiver<Request>,
    cache: Cache,
    upstreams: Upstreams,
}

impl Resolver {
    pub fn new(
        queue_receiver: Receiver<Request>,
        resolver_settings: &ResolverSettings,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            blocklist: Blocklist::new(),
            queue_receiver,
            cache: Cache::new(&resolver_settings.cache),
            upstreams: Upstreams::new(&resolver_settings.upstreams),
        })
    }

    pub async fn start(mut self: Self) {
        while let Some(request) = self.queue_receiver.recv().await {
            self.process_message(request).await;
        }
    }

    async fn process_message(&self, request: Request) {
        let request_domain = request.message.qname_to_string();

        if self.blocklist.contains(&request_domain) {
            println!("Domain {} is blocked.", request_domain);
            return;
        }

        if let Some(cached_response) = self.cache.query(&request_domain) {
            println!("Cache hit for domain: {}", request_domain);
            // Asynchronously send cached_response back to the client
            // Ensure send_response is awaited
            //if let Err(e) = request.send_response(&cached_response.serialize()).await {
            //    eprintln!("Failed to send cached response: {}", e);
            //}
            return;
        }

        if let Some(response) = self.upstreams.query(&request_domain) {
            println!("Response from upstream for domain: {}", request_domain);
            // Asynchronously send response back to the client
            // Ensure send_response is awaited
            //if let Err(e) = request.send_response(&response.serialize()).await {
            //    eprintln!("Failed to send upstream response: {}", e);
            //}
            return;
        }

        // Handling domain not found
        println!("Domain {} not found.", request_domain);
        let response = DNSMessage::new_not_found_response(&request.message);
        let response_data = response.serialize();
        if let Err(e) = request.send_response(&response_data).await {
            eprintln!("Failed to send not found response: {}", e);
        }
    }
}
