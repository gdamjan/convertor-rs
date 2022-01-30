use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn list_styles(styles: impl Read) -> std::io::Result<()> {
    let reader = BufReader::new(styles);
    let mut reader = Reader::from_reader(reader);

    // let mut count = 0;
    // let mut txt = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
              println!("{:?}", String::from_utf8_lossy(e.name()));
            },
            // Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let file = File::open("problem.odt")?;
    let mut zip = zip::ZipArchive::new(file)?;

    let styles = zip.by_name("styles.xml")?;
    list_styles(styles);

    let contents = zip.by_name("contents.xml")?;

    Ok(())
}
