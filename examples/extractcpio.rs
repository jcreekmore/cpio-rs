// Extract a file from a CPIO archive.

extern crate cpio;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let filename = std::env::args().nth(2).unwrap();
    let output = std::env::args().nth(3).unwrap();
    let mut file = std::fs::File::open(path).unwrap();
    loop {
        let reader = cpio::NewcReader::new(file).unwrap();
        if reader.entry().is_trailer() {
            break;
        }

        if filename == reader.entry().name() {
            println!(
                "{} ({} bytes)",
                reader.entry().name(),
                reader.entry().file_size()
            );

            let out = std::fs::File::create(&output).unwrap();
            file = reader.to_writer(out).unwrap();
        } else {
            file = reader.skip().unwrap();
        }
    }
}
