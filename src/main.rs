#![warn(clippy::pedantic, clippy::nursery, clippy::all)]
#![allow(clippy::multiple_crate_versions, clippy::module_name_repetitions)]

#[tokio::main]
async fn main() {
	std::env::set_var("RUST_LOG", "");
	std::env::set_var("RUST_BACKTRACE", "full");
	std::env::set_var("log_level", "debug");
	tracing_subscriber::fmt().init();

	let uri = "wss://ixwlfx3jfffjx25pq5supc3lojsg6yyd53bncnfbmlpi5ityqlavonad.onion/ws".parse::<hyper::Uri>().unwrap();
	let host = uri.host().unwrap();
	let https = uri.scheme() == Some(&hyper::http::uri::Scheme::HTTPS);
	let port = match uri.port_u16() {
		Some(port) => port,
		_ if https => 443,
		_ => 80,
	};

	let mut tor_config = arti_client::config::TorClientConfigBuilder::default();
	tor_config.address_filter().allow_onion_addrs(true);
	tor_config.storage().cache_dir(arti_client::config::CfgPath::new("./temp/arti/cache".to_string()));
	tor_config.storage().state_dir(arti_client::config::CfgPath::new("./temp/arti/data".to_string()));
	let tor_config = tor_config.build().unwrap();
	let tor_client = arti_client::TorClient::create_bootstrapped(tor_config).await.unwrap();
	let stream = tor_client.connect((host, port)).await.unwrap();

	balens_log::log(balens_log::Level::Info, format!("connected to {uri} via tor"));

	balens_log::log(balens_log::Level::Info, "Beginning TLS handshake...".to_string());
	// tls
	let alpn_protocols = vec!["http/1.1", "h2", "webrtc", "h3"];
	let tls_connector = native_tls::TlsConnector::builder().request_alpns(&alpn_protocols).danger_accept_invalid_certs(true).build().unwrap();

	let tls_stream = tokio_native_tls::TlsConnector::from(tls_connector.clone()).connect(host, stream).await.unwrap();

	balens_log::log(balens_log::Level::Info, "...TLS handshake complete".to_string());
	balens_log::log(balens_log::Level::Info, "Connecting to websocket...".to_string());

	let (_, response) = tokio_tungstenite::client_async(&uri.to_string(), tls_stream).await.unwrap();
	balens_log::log(balens_log::Level::Info, format!("Connected to websocket: {response:?}"));
}
