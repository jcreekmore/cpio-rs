//! A library for reading and writing [CPIO
//! archives](https://en.wikipedia.org/wiki/Cpio).
//!
//! CPIO archives can be in any of several
//! [formats](https://www.gnu.org/software/cpio/manual/cpio.html#format).  For
//! now, this library only supports the `newc` (SVR4) format.

use std::io;
use std::iter::Iterator;

pub mod newc;
pub use newc::Builder as NewcBuilder;
pub use newc::Reader as NewcReader;

/// Creates a new CPIO archive.
pub fn write_cpio<I, RS, W>(inputs: I, output: W) -> io::Result<W>
where
    I: Iterator<Item = (NewcBuilder, RS)> + Sized,
    RS: io::Read + io::Seek,
    W: io::Write,
{
    let output = inputs
        .enumerate()
        .fold(Ok(output), |output, (idx, (builder, mut input))| {
            // If the output is valid, try to write the next input file
            output.and_then(move |output| {
                // Grab the length of the input file
                let len = input.seek(io::SeekFrom::End(0))?;
                input.seek(io::SeekFrom::Start(0))?;

                // Create our writer fp with a unique inode number
                let mut fp = builder.ino(idx as u32).write(output, len as u32);

                // Write out the file
                io::copy(&mut input, &mut fp)?;

                // And finish off the input file
                fp.finish()
            })
        })?;

    newc::trailer(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_multi_file() {
        // Set up our input files
        let mut input = vec![
            (
                NewcBuilder::new("./hello_world")
                    .uid(1000)
                    .gid(1000)
                    .mode(0o100644),
                Cursor::new("Hello, World".to_string()),
            ),
            (
                NewcBuilder::new("./hello_world2")
                    .uid(1000)
                    .gid(1000)
                    .mode(0o100644),
                Cursor::new("Hello, World 2".to_string()),
            ),
        ];

        // Set up our output file
        let output = Cursor::new(vec![]);

        // Write out the CPIO archive
        let _ = write_cpio(input.drain(..), output).unwrap();
    }
}
