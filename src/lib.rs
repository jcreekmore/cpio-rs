use std::io;

pub mod newc;
pub use newc::Builder as NewcBuilder;

pub fn write_cpio<RS, W>(mut inputs: Vec<(NewcBuilder, RS)>, output: W) -> io::Result<W>
    where RS: io::Read + io::Seek,
          W: io::Write
{
    let output = try!(inputs.drain(..)
        .enumerate()
        .fold(Ok(output), |output, (idx, (builder, mut input))| {

            // If the output is valid, try to write the next input file
            output.and_then(move |output| {

                // Grab the length of the input file
                let len = try!(input.seek(io::SeekFrom::End(0)));
                try!(input.seek(io::SeekFrom::Start(0)));

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
        let input = vec![
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
        let _ = write_cpio(input, output).unwrap();
    }
}
