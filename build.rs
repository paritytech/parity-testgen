extern crate syntex;
extern crate serde_codegen;

use std::env;
use std::path::Path;

fn main() {
	let out_dir = env::var_os("OUT_DIR").unwrap();

	let src = Path::new("src/rpc/response.rs.in");
	let dst = Path::new(&out_dir).join("response.rs");

	expand(&src, &dst);

	let src = Path::new("src/action/mod.rs.in");
	let dst = Path::new(&out_dir).join("action.rs");

	expand(&src, &dst);
}

fn expand(src: &Path, dst: &Path) {
	let mut registry = syntex::Registry::new();

	serde_codegen::register(&mut registry);
	registry.expand("", src, dst).unwrap();
}