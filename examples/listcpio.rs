// Lists files in a CPIO archive.

extern crate cpio;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut file = std::fs::File::open(path).unwrap();
    loop {
        let reader = cpio::NewcReader::new(file).unwrap();
        if reader.entry().is_trailer() {
            break;
        }
        println!(
            "{} ({} bytes)",
            reader.entry().name(),
            reader.entry().file_size()
        );
        file = reader.finish().unwrap();
    }
}
