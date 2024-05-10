// Create a CPIO archive from filenames passed through stdin.

use cpio::{newc::trailer, NewcBuilder};
use std::io::{self, stdout};

fn main() {
    // Set up our input files
    let data1: &[u8] = b"Hello, World";
    let length1 = data1.len() as u32;
    let mut input1 = io::Cursor::new(data1);

    let data2: &[u8] = b"Hello, World 2";
    let length2 = data2.len() as u32;
    let mut input2 = io::Cursor::new(data2);

    // Set up our output file
    let output = stdout();

    // Set up the descriptor of our input file
    let b = NewcBuilder::new("./hello_world")
        .ino(1)
        .uid(1000)
        .gid(1000)
        .mode(0o100644);
    // and get a writer for that input file
    let mut writer = b.write(output, length1);

    // Copy the input file into our CPIO archive
    io::copy(&mut input1, &mut writer).unwrap();
    let output = writer.finish().unwrap();

    // Set up the descriptor of an empty directory
    let b = NewcBuilder::new("./empty_dir")
        .ino(2)
        .uid(1000)
        .gid(1000)
        .mode(0o000755)
        .set_mode_file_type(cpio::newc::ModeFileType::Directory);
    let writer = b.write(output, 0);
    let output = writer.finish().unwrap();

    // Set up the descriptor of our second input file
    let b = NewcBuilder::new("./hello_world2")
        .ino(3)
        .uid(1000)
        .gid(1000)
        .mode(0o100644);
    // and get a writer for that input file
    let mut writer = b.write(output, length2);

    // Copy the second input file into our CPIO archive
    io::copy(&mut input2, &mut writer).unwrap();
    let output = writer.finish().unwrap();

    let data: &[u8] = b"./hello_world2";
    let length = data.len() as u32;
    let mut input = io::Cursor::new(data);

    // Set up the descriptor for a symlink
    let b = NewcBuilder::new("./hello-link")
        .ino(4)
        .uid(1000)
        .gid(1000)
        .mode(0o100644)
        .set_mode_file_type(cpio::newc::ModeFileType::Symlink);
    let mut writer = b.write(output, length);
    io::copy(&mut input, &mut writer).unwrap();
    let output = writer.finish().unwrap();

    // Finish up by writing the trailer for the archive
    let _ = trailer(output).unwrap();
}
