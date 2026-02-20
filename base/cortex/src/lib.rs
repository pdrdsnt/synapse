pub mod cortex;
pub mod integration;
pub mod types;
use std::{num::NonZeroUsize, str::FromStr};

use alloy::providers::{Provider, ProviderBuilder};
use alloy::{
    rpc::client::RpcClient,
    transports::{http::Http, layers::FallbackLayer},
};
use tower::ServiceBuilder;
use url::Url;

use crate::cortex::WsProvider;

pub fn generate_fallback_p(urls: Vec<String>) -> Option<impl Provider + Clone> {
    generate_fallback_provider(urls)
}

pub fn generate_fallback_provider(urls: Vec<String>) -> Option<WsProvider> {
    if urls.is_empty() {
        return None;
    }

    let mut trnsports = Vec::new();
    for _url in &urls {
        if let Ok(url) = Url::from_str(_url) {
            let http = Http::new(url);
            trnsports.push(http);
        }
    }

    let count = NonZeroUsize::try_from(trnsports.len()).unwrap();
    let fallback_layer = FallbackLayer::default().with_active_transport_count(count);
    let service = ServiceBuilder::new()
        .layer(fallback_layer)
        .service(trnsports);

    let client = RpcClient::builder().transport(service, false);
    Some(ProviderBuilder::new().connect_client(client))
}
