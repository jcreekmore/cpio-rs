use std::io::{self, Write};

const HEADER_LEN: usize = 110;

pub struct Builder {
    name: String,
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    nlink: u32,
    mtime: u32,
    dev_major: u32,
    dev_minor: u32,
    rdev_major: u32,
    rdev_minor: u32,
}

pub struct Writer<W: Write> {
    inner: W,
    written: u32,
    file_size: u32,
    header_size: usize,
    header: Vec<u8>,
}

fn pad(len: usize) -> Option<Vec<u8>> {
    // pad out to a multiple of 4 bytes
    let overhang = len % 4;
    if overhang != 0 {
        let repeat = 4 - overhang;
        Some(vec![0u8; repeat])
    } else {
        None
    }
}

impl Builder {
    pub fn new(name: &str) -> Builder {
        Builder {
            name: name.to_string(),
            ino: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            nlink: 1,
            mtime: 0,
            dev_major: 0,
            dev_minor: 0,
            rdev_major: 0,
            rdev_minor: 0,
        }
    }

    pub fn ino(mut self, ino: u32) -> Builder {
        self.ino = ino;
        self
    }

    pub fn mode(mut self, mode: u32) -> Builder {
        self.mode = mode;
        self
    }

    pub fn uid(mut self, uid: u32) -> Builder {
        self.uid = uid;
        self
    }

    pub fn gid(mut self, gid: u32) -> Builder {
        self.gid = gid;
        self
    }

    pub fn nlink(mut self, nlink: u32) -> Builder {
        self.nlink = nlink;
        self
    }

    pub fn mtime(mut self, mtime: u32) -> Builder {
        self.mtime = mtime;
        self
    }

    pub fn dev_major(mut self, dev_major: u32) -> Builder {
        self.dev_major = dev_major;
        self
    }

    pub fn dev_minor(mut self, dev_minor: u32) -> Builder {
        self.dev_minor = dev_minor;
        self
    }

    pub fn rdev_major(mut self, rdev_major: u32) -> Builder {
        self.rdev_major = rdev_major;
        self
    }

    pub fn rdev_minor(mut self, rdev_minor: u32) -> Builder {
        self.rdev_minor = rdev_minor;
        self
    }

    pub fn write<W: Write>(self, w: W, file_size: u32) -> Writer<W> {
        let header = self.into_header(file_size);

        Writer {
            inner: w,
            written: 0,
            file_size: file_size,
            header_size: header.len(),
            header: header,
        }
    }

    fn into_header(self, file_size: u32) -> Vec<u8> {
        let mut header = Vec::with_capacity(HEADER_LEN);

        // char    c_magic[6];
        header.extend("070701".to_string().as_bytes());
        // char    c_ino[8];
        header.extend(format!("{:08x}", self.ino).as_bytes());
        // char    c_mode[8];
        header.extend(format!("{:08x}", self.mode).as_bytes());
        // char    c_uid[8];
        header.extend(format!("{:08x}", self.uid).as_bytes());
        // char    c_gid[8];
        header.extend(format!("{:08x}", self.gid).as_bytes());
        // char    c_nlink[8];
        header.extend(format!("{:08x}", self.nlink).as_bytes());
        // char    c_mtime[8];
        header.extend(format!("{:08x}", self.mtime).as_bytes());
        // char    c_filesize[8];
        header.extend(format!("{:08x}", file_size).as_bytes());
        // char    c_devmajor[8];
        header.extend(format!("{:08x}", self.dev_major).as_bytes());
        // char    c_devminor[8];
        header.extend(format!("{:08x}", self.dev_minor).as_bytes());
        // char    c_rdevmajor[8];
        header.extend(format!("{:08x}", self.rdev_major).as_bytes());
        // char    c_rdevminor[8];
        header.extend(format!("{:08x}", self.rdev_minor).as_bytes());
        // char    c_namesize[8];
        let name_len = self.name.len() + 1;
        header.extend(format!("{:08x}", name_len).as_bytes());
        // char    c_check[8];
        header.extend(format!("{:08x}", 0).as_bytes());

        // append the name to the end of the header
        header.extend(self.name.as_bytes());
        header.push(0u8);

        // pad out to a multiple of 4 bytes
        if let Some(pad) = pad(HEADER_LEN + name_len) {
            header.extend(pad);
        }

        header
    }
}

impl<W: Write> Writer<W> {
    pub fn finish(mut self) -> io::Result<W> {
        try!(self.do_finish());
        Ok(self.inner)
    }

    fn try_write_header(&mut self) -> io::Result<()> {
        if self.header.len() != 0 {
            try!(self.inner.write_all(&self.header));
            self.header.truncate(0);
        }
        Ok(())
    }

    fn do_finish(&mut self) -> io::Result<()> {
        try!(self.try_write_header());

        if self.written == self.file_size {
            if let Some(pad) = pad(self.header_size + self.file_size as usize) {
                try!(self.inner.write(&pad));
                try!(self.inner.flush());
            }
        }

        Ok(())
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.written + buf.len() as u32 <= self.file_size {
            try!(self.try_write_header());

            let n = try!(self.inner.write(buf));
            self.written += n as u32;
            Ok(n)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                               "trying to write more than the specified file size"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub fn trailer<W: Write>(w: W) -> io::Result<()> {
    let b = Builder::new("TRAILER!!!").nlink(0);
    let writer = b.write(w, 0);
    let _ = try!(writer.finish());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{copy, Cursor};

    #[test]
    fn test_single_file() {
        // Set up our input file
        let data = "Hello, World".to_string();
        let length = data.len() as u32;
        let mut input = Cursor::new(data);

        // Set up our output file
        let output = Cursor::new(vec![]);

        // Set up the descriptor of our input file
        let b = Builder::new("./hello_world");
        // and get a writer for that input file
        let mut writer = b.write(output, length);

        // Copy the input file into our CPIO archive
        copy(&mut input, &mut writer).unwrap();
        let output = writer.finish().unwrap();

        // Finish up by writing the trailer for the archive
        trailer(output).unwrap();
    }

    #[test]
    fn test_multi_file() {
        // Set up our input files
        let data1 = "Hello, World".to_string();
        let length1 = data1.len() as u32;
        let mut input1 = Cursor::new(data1);

        let data2 = "Hello, World 2".to_string();
        let length2 = data2.len() as u32;
        let mut input2 = Cursor::new(data2);

        // Set up our output file
        let output = Cursor::new(vec![]);

        // Set up the descriptor of our input file
        let b = Builder::new("./hello_world")
            .ino(1)
            .uid(1000)
            .gid(1000)
            .mode(0o100644);
        // and get a writer for that input file
        let mut writer = b.write(output, length1);

        // Copy the input file into our CPIO archive
        copy(&mut input1, &mut writer).unwrap();
        let output = writer.finish().unwrap();

        // Set up the descriptor of our second input file
        let b = Builder::new("./hello_world2")
            .ino(2)
            .uid(1000)
            .gid(1000)
            .mode(0o100644);
        // and get a writer for that input file
        let mut writer = b.write(output, length2);

        // Copy the second input file into our CPIO archive
        copy(&mut input2, &mut writer).unwrap();
        let output = writer.finish().unwrap();

        // Finish up by writing the trailer for the archive
        trailer(output).unwrap();
    }
}
