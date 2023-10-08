// Create a CPIO archive from filenames passed through stdin.

use cpio::{NewcBuilder, write_cpio};
use std::fs::File;
use std::io::{self, BufRead, stdin, stdout};

fn load_file(path: &str) -> io::Result<(NewcBuilder, File)> {
	let builder = NewcBuilder::new(path)
		.uid(1000)
		.gid(1000)
		.mode(0o100644);
		
	File::open(path)
		.map(|fp| (builder, fp))
}

fn main() {
	let stdin = stdin();
	let inputs = stdin
		.lock()
		.lines()
		.map(|path| load_file(&path.unwrap()).unwrap());
		
	write_cpio(inputs, stdout()).unwrap();
}
