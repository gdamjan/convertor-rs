use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn list_styles(styles: impl Read) -> std::io::Result<()> {
    let reader = BufReader::new(styles);
    let mut reader = Reader::from_reader(reader);

    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => match e.name() {
              b"style:text-properties" => {
                let v = e.attributes()
                    .filter_map(|a| a.ok().filter(|a| a.key == b"style:font-name").map(|a| a.value))
                    .collect::<Vec<_>>();
                if v.len() > 0 {
                  let font_name = String::from_utf8_lossy(&v[0]);
                  println!("{}: {}", String::from_utf8_lossy(e.name()), font_name);
                }
              },
              _ => ()
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
    let file = File::open("fixtures/problem.odt")?;
    let mut zip = zip::ZipArchive::new(file)?;

    let styles = zip.by_name("styles.xml")?;
    list_styles(styles)?;

    //let contents = zip.by_name("contents.xml")?;

    Ok(())
}
