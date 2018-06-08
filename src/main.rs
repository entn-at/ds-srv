#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate simplelog;

use std::thread;
use std::sync::mpsc::sync_channel;

mod args;
use args::ArgsParser;

mod http;
use http::th_http_listener;

mod inference;
use inference::th_inference;

fn main() {
	let rc = ArgsParser::from_cli();

	let log_level = rc.verbosity_level.into();
	let _ = simplelog::TermLogger::init(log_level, simplelog::Config::default());

	debug!("Parsed all CLI args: {:?}", rc);

	let (tx_audio, rx_audio) = sync_channel(0);
	let (tx_string, rx_string) = sync_channel(0);

	let mut threads = Vec::new();
	let rc_inference = rc.clone();
	let thread_inference = thread::Builder::new().name("InferenceService".to_string()).spawn(move || {
		th_inference(
			rc_inference.model,
			rc_inference.alphabet,
			rc_inference.lm,
			rc_inference.trie,
			rx_audio,
			tx_string,
			rc_inference.dump_dir
		);
	});
	threads.push(thread_inference);

	let rc_http = rc.clone();
	let thread_http = thread::Builder::new().name("HttpService".to_string()).spawn(move || {
		th_http_listener(rc_http.http_ip, rc_http.http_port, tx_audio, rx_string);
	});
	threads.push(thread_http);

	println!("Started all thread.");

	for hdl in threads {
		if hdl.is_ok() {
			hdl.unwrap().join().unwrap();
		}
	}
}
