use std::env::temp_dir;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};

use fsdb::InMemoryTable;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream?;
        println!("Connection established!");
        handle_connection(stream)?;
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = stream.try_clone()?;
    let buf_reader = BufReader::new(&reader);
    for line in buf_reader.lines() {
        println!("Request: {:#?}", &line);
        let Ok(line) = line else { continue };
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let response = match parts[..] {
            ["create", table_name, value_type] => {
                let in_memory_table = InMemoryTable::new(value_type, &temp_dir().join(table_name));
                in_memory_table.flush()?;
                format!("ok: {:#?} \n", &parts)
            }
            ["insert", table_name, key, value] => {
                let mut in_memory_table = match InMemoryTable::load(&temp_dir().join(table_name)) {
                    Ok(it) => it,
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        stream.write_all(b"Not found!\n")?;
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                in_memory_table.insert(key.to_owned(), value.to_owned())?;
                format!("ok: {:#?} \n", &parts)
            }
            ["metadata", table_name] => {
                let in_memory_table = match InMemoryTable::load(&temp_dir().join(table_name)) {
                    Ok(it) => it,
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        stream.write_all(b"Not found!\n")?;
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                format!("{table_name} type {0} \n", in_memory_table.metadata())
            }
            ["select", table_name, key] => {
                match InMemoryTable::load(&temp_dir().join(table_name)) {
                    Ok(it) => it,
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        stream.write_all(b"Not found!\n")?;
                        continue;
                    }
                    Err(err) => return Err(err),
                }
                .get(key)
                .map_or_else(
                    || "Failed!\n".to_owned(),
                    |value| format!("{key}: {value} \n"),
                )
            }
            ["remove", table_name, key] => {
                let mut in_memory_table = match InMemoryTable::load(&temp_dir().join(table_name)) {
                    Ok(it) => it,
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        stream.write_all(b"Not found!\n")?;
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                match in_memory_table.remove(&key.to_owned()) {
                    Ok(()) => format!("ok: {:#?} \n", &parts),
                    Err(err) if err.kind() == ErrorKind::NotFound => "Not found!\n".to_owned(),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            ["exit"] => {
                stream.write_all(b"Goodbye\n")?;
                break;
            }
            _ => "Bad command\n".to_owned(),
        };
        stream.write_all(response.as_bytes())?;
    }
    println!("Connection closed!");
    Ok(())
}
