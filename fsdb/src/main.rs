use std::env::temp_dir;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use fsdb::InMemoryDB;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?;
        println!("Connection established!");
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let inner = &stream.try_clone()?;
    let buf_reader = BufReader::new(inner);
    for line in buf_reader.lines() {
        println!("Request: {line:#?}");
        // let response = format!("ok: {line:#?} \n");
        // stream.write_all(response.as_bytes())?;
        let line = &line?;
        if line.starts_with("create ") {
            let values: Vec<&str> = line.split_whitespace().skip(1).take(2).collect();
            let (Some(table_name), Some(value_type)) = (values.first(), values.get(1)) else {
                continue;
            };
            let db = InMemoryDB::new(value_type, &temp_dir().join(table_name));
            db.flush()?;
            let response = format!("ok: {values:#?} \n");
            stream.write_all(response.as_bytes())?;
        }
        if line.starts_with("insert ") {
            let values: Vec<&str> = line.split_whitespace().skip(1).take(3).collect();
            let (Some(table_name), Some(key), Some(value)): (
                Option<&str>,
                Option<&str>,
                Option<&str>,
            ) = (
                values.first().copied(),
                values.get(1).copied(),
                values.get(2).copied(),
            ) else {
                continue;
            };
            let mut db = InMemoryDB::load(&temp_dir().join(table_name))?;
            db.insert(key.to_owned(), value.to_owned())?;
            let response = format!("ok: {values:#?} \n");
            stream.write_all(response.as_bytes())?;
        }
        if line.starts_with("metadata ") {
            let values: Vec<&str> = line.split_whitespace().skip(1).take(1).collect();
            let Some(table_name): Option<&str> = values.first().copied() else {
                continue;
            };
            let db = InMemoryDB::load(&temp_dir().join(table_name))?;
            let response = format!("{table_name} type {0} \n", db.metadata());
            stream.write_all(response.as_bytes())?;
        }
        if line.starts_with("select ") {
            let values: Vec<&str> = line.split_whitespace().skip(1).take(2).collect();
            let (Some(table_name), Some(key)): (Option<&str>, Option<&str>) =
                (values.first().copied(), values.get(1).copied())
            else {
                continue;
            };
            let db = InMemoryDB::load(&temp_dir().join(table_name))?;
            let response = if let Some(value) = db.get(&key.to_owned()) {
                format!("{key}: {value} \n")
            } else {
                "Failed!\n".to_owned()
            };
            stream.write_all(response.as_bytes())?;
        }
        if line.starts_with("remove ") {
            let values: Vec<&str> = line.split_whitespace().skip(1).take(2).collect();
            let (Some(table_name), Some(key)): (Option<&str>, Option<&str>) =
                (values.first().copied(), values.get(1).copied())
            else {
                continue;
            };

            let mut db = InMemoryDB::load(&temp_dir().join(table_name))?;
            let entry = db.contains_key(&key.to_owned());

            let response = if entry {
                db.remove(&key.to_owned())?;
                format!("ok: {values:#?} \n")
            } else {
                "Failed!\n".to_owned()
            };
            stream.write_all(response.as_bytes())?;
        }

        if line == "exit" {
            let response = "We will perish\n";
            stream.write_all(response.as_bytes())?;
            break;
        }
    }

    Ok(())
}
