//! RPC client for communicating with parity.
use ethkey::Address;
use hyper::{self, Client};
use serde::Deserialize;
use serde_json;

use std::io::Read;
use std::net::Ipv4Addr;

use self::response::Response;

mod response;

/// Errors which can be encountered.
pub enum Error {
	Hyper(hyper::Error),
	Io(::std::io::Error),
	Serde(serde_json::Error),
}

// could make this configurable at some point
const PORT: u16 = 8545;

pub struct RpcClient {
	http_client: Client,
	server_url: String,
	req_id: usize
}

impl RpcClient {
	/// Create a new RpcClient
	pub fn new(ip: Ipv4Addr) -> Self {
		RpcClient {
			http_client: Client::new(),
			server_url: format!("http://{}:{}", ip, PORT),
			req_id: 0,
		}
	}

	/// Get the balance (in wei) of the given account at the latest
	/// block.
	pub fn balance(&mut self, address: Address) -> Result<usize, Error> {
		self.req_id += 1;
		let req = format!(r#"{{
			"jsonrpc": "2.0",
			"method": "eth_getBalance",
			"params": [
				"0x{}",
				"latest"
			],
			"id": {}
		}}"#, address, self.req_id);

		let mut res_str = String::new();
		let mut res = try!(self.http_client.post(&self.server_url).body(&req).send().map_err(Error::Hyper));
		try!(res.read_to_string(&mut res_str).map_err(Error::Io));

		Ok(try!(Response::from_json(&res_str)).result())
	}
}