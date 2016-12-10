use std::io;
use std::iter::Iterator;

pub mod newc;
pub use newc::Builder as NewcBuilder;

fn length<R: io::Read + io::Seek>(s: &mut R) -> io::Result<u64> {
    // Grab the current stream offset
    let offset = try!(s.seek(io::SeekFrom::Current(0)));
    // Seek to the end and get the current offset as the length
    let len = try!(s.seek(io::SeekFrom::End(0)));
    // Go back to the previous stream offset
    try!(s.seek(io::SeekFrom::Start(offset)));

    Ok(len)
}

pub fn write_cpio<I, RS, W>(inputs: I, output: W) -> io::Result<W>
    where I: Iterator<Item = (NewcBuilder, RS)> + Sized,
          RS: io::Read + io::Seek,
          W: io::Write
{
    let output = try!(inputs.enumerate()
        .fold(Ok(output), |output, (idx, (builder, mut input))| {

            // If the output is valid, try to write the next input file
            output.and_then(move |output| {

                // Grab the length of the input file
                let len = try!(length(&mut input));

                // Create our writer fp with a unique inode number
                let mut fp = builder.ino(idx as u32)
                    .write(output, len as u32);

                // Write out the file
                try!(io::copy(&mut input, &mut fp));

                // And finish off the input file
                fp.finish()
            })
        }));

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
            (NewcBuilder::new("./hello_world")
                .uid(1000)
                .gid(1000)
                .mode(0o100644), Cursor::new("Hello, World".to_string())),
            (NewcBuilder::new("./hello_world2")
                .uid(1000)
                .gid(1000)
                .mode(0o100644), Cursor::new("Hello, World 2".to_string())),
        ];

        // Set up our output file
        let output = Cursor::new(vec![]);

        // Write out the CPIO archive
        let _ = write_cpio(input.drain(..), output).unwrap();
    }
}
