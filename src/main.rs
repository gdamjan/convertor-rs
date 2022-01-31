use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesEnd, BytesStart};

use std::fs::File;
use std::io::{Cursor, BufReader, BufRead, Read, Write};
use std::iter;


fn convert_styles(styles: impl Read, converted: &mut impl Write) -> std::io::Result<()> {
  let reader = BufReader::new(styles);
  let mut reader = Reader::from_reader(reader);

  let mut buf = Vec::new();
  loop {
    buf.clear();
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
  }

  Ok(())
}

fn convert_body<B, W>(reader: &mut Reader<B>, writer: &mut Writer<W>)
where
    B: BufRead,
    W: Write,
{
  let mut buf = Vec::new();
  loop {
    buf.clear();
    match reader.read_event(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(Event::Eof) => break, // panic ? we really should see End of office:body
      Ok(Event::Text(ref e)) => {
        println!("{}", e.unescape_and_decode(&reader).unwrap())
      },
      Ok(Event::End(ref e)) => {
        assert!(writer.write_event(Event::End(e.clone())).is_ok());
        if e.name() ==  b"office:body" { break };
      },
      Ok(e) => assert!(writer.write_event(&e).is_ok()),
    }
  }
}

fn convert_content(content: impl Read, converted: &mut impl Write) -> std::io::Result<()> {
  let reader = BufReader::new(content);
  let mut reader = Reader::from_reader(reader);
  reader.trim_text(true);

  let mut writer = Writer::new(converted);
  let mut buf = Vec::new();
  loop {
    buf.clear();
    match reader.read_event(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(Event::Eof) => break,
      Ok(Event::Start(ref e)) => {
        assert!(writer.write_event(Event::Start(e.clone())).is_ok());
        if e.name() == b"office:body" {
          convert_body(&mut reader, &mut writer);
        };
      },
      Ok(e) => assert!(writer.write_event(&e).is_ok()),
    }
  }

  Ok(())
}

fn main() -> std::io::Result<()> {
  let input = File::open("fixtures/problem.odt")?;
  let mut input = zip::ZipArchive::new(input)?;

  let output = File::create("fixtures/solved.odt")?;
  let mut output = zip::ZipWriter::new(output);

  let styles = input.by_name("styles.xml")?;
  output.start_file("styles.xml", zip::write::FileOptions::default())?;
  convert_styles(styles, &mut output)?;

  let content = input.by_name("content.xml")?;
  output.start_file("content.xml", zip::write::FileOptions::default())?;
  convert_content(content, &mut output)?;

  output.finish()?;
  Ok(())
}
