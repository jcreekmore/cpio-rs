use std::io::{self, Write};

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
        let mut header = Vec::with_capacity(110);

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
        let name_len = self.name.len();
        header.extend(format!("{:08x}", name_len).as_bytes());
        // char    c_check[8];
        header.extend(format!("{:08x}", 0).as_bytes());

        // append the name to the end of the header
        header.extend(self.name.as_bytes());

        // pad out to a multiple of 4 bytes
        if let Some(pad) = pad(name_len) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_single_file() {
         let data = "Hello, World".to_string();

         let fp = File::create("/tmp/test_single.cpio").unwrap();

         let b = Builder::new("/tmp/hello_world");
         let mut writer = b.write(fp, data.len() as u32);
         writer.write_all(data.as_bytes()).unwrap();
         writer.flush().unwrap();
         let _ = writer.finish().unwrap();
    }

    #[test]
    fn test_multi_file() {
         let data = "Hello, World".to_string();

         let fp = File::create("/tmp/test_multi.cpio").unwrap();

         let b = Builder::new("/tmp/hello_world")
                    .ino(1);
         let mut writer = b.write(fp, data.len() as u32);
         writer.write_all(data.as_bytes()).unwrap();
         writer.flush().unwrap();
         let fp = writer.finish().unwrap();

         let b = Builder::new("/tmp/hello_world2")
                    .ino(2);
         let mut writer = b.write(fp, data.len() as u32);
         writer.write_all(data.as_bytes()).unwrap();
         writer.flush().unwrap();
         let _ = writer.finish().unwrap();
    }
}
