use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

/// Find all styles that use the YUSCII fonts, and convert them to use normal fonts
/// Keep track what styles were converted and which font was used
fn convert_styles(styles: impl Read, converted: &mut impl Write) -> std::io::Result<()> {
  let reader = BufReader::new(styles);
  let mut reader = Reader::from_reader(reader);

  let mut writer = Writer::new(converted);
  let mut buf = Vec::new();
  loop {
    buf.clear();
    match reader.read_event(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(Event::Eof) => break,
      Ok(Event::Empty(ref e)) if e.name() == b"style:text-properties" => {
        let font_name = e
          .attributes()
          .filter_map(|res| res.ok())
          .find(|a| a.key == b"style:font-name");

        if let Some(font_name) = font_name {
          let font_name = String::from_utf8_lossy(&font_name.value);
          let e_name = String::from_utf8_lossy(e.name());
          println!("{}: {}", e_name, font_name);
        }
      }
      Ok(e) => assert!(writer.write_event(&e).is_ok()),
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
        println!("{}", e.unescape_and_decode(&reader).unwrap());
        assert!(writer.write_event(Event::Text(e.clone())).is_ok());
      }
      Ok(Event::End(ref e)) => {
        assert!(writer.write_event(Event::End(e.clone())).is_ok());
        if e.name() == b"office:body" {
          break;
        };
      }
      Ok(e) => assert!(writer.write_event(&e).is_ok()),
    }
  }
}

fn convert_content(content: impl Read, converted: &mut impl Write) -> std::io::Result<()> {
  let reader = BufReader::new(content);
  let mut reader = Reader::from_reader(reader);
  // reader.trim_text(true); // FIXME: remove from the final version

  let mut writer = Writer::new(converted);
  let mut buf = Vec::new();
  loop {
    buf.clear();
    match reader.read_event(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(Event::Eof) => break,
      Ok(Event::Start(ref e)) if e.name() == b"office:body" => {
        assert!(writer.write_event(Event::Start(e.clone())).is_ok());
        convert_body(&mut reader, &mut writer);
      }
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

  for i in 0..input.len() {
    let file = input.by_index(i)?;
    if ["styles.xml", "content.xml"].contains(&file.name()) {
      continue;
    }
    output.raw_copy_file(file)?;
  }

  let styles = input.by_name("styles.xml")?;
  let options = zip::write::FileOptions::default().compression_method(styles.compression());
  output.start_file("styles.xml", options)?;
  convert_styles(styles, &mut output)?;

  let content = input.by_name("content.xml")?;
  let options = zip::write::FileOptions::default().compression_method(content.compression());
  output.start_file("content.xml", options)?;
  convert_content(content, &mut output)?;

  output.finish()?;
  Ok(())
}
