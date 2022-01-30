use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn list_styles_with_fonts(styles: impl Read) -> std::io::Result<()> {
    let reader = BufReader::new(styles);
    let mut reader = Reader::from_reader(reader);

    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => match e.name() {
                b"style:text-properties" => {
                    let v = e
                        .attributes()
                        .filter_map(|a| a.ok().filter(|a| a.key == b"style:font-name").map(|a| a.value))
                        .collect::<Vec<_>>();
                    if v.len() > 0 {
                        let font_name = String::from_utf8_lossy(&v[0]);
                        println!("{}: {}", String::from_utf8_lossy(e.name()), font_name);
                    }
                }
                _ => (),
            },
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }

    Ok(())
}

fn inside_body<B>(reader: &mut Reader<B>)
where
    B: BufRead,
{
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Text(ref e)) => {
                println!("{}", e.unescape_and_decode(&reader).unwrap())
            }
            Ok(Event::End(ref e)) => match e.name() {
                b"office:body" => break,
                _ => (),
            },
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
}

fn print_content_body(content: impl Read) -> std::io::Result<()> {
    let reader = BufReader::new(content);
    let mut reader = Reader::from_reader(reader);

    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"office:body" => inside_body(&mut reader),
                _ => (),
            },
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
    list_styles_with_fonts(styles)?;

    let content = zip.by_name("content.xml")?;
    print_content_body(content)?;

    Ok(())
}
