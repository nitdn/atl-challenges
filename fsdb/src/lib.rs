use std::{
    collections::HashMap,
    fs::{self, read_dir},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, Debug)]
pub struct InMemoryTable {
    value_type: String,
    base_path: PathBuf,
    data: HashMap<String, String>,
}

fn typeof_path(base_path: &Path) -> PathBuf {
    base_path.join("metadata/type")
}

fn data_path(base_path: &Path) -> PathBuf {
    base_path.join("data")
}

impl InMemoryTable {
    #[must_use]
    pub fn new(value_type: &str, base_path: &Path) -> Self {
        let hash_map = HashMap::new();
        Self {
            value_type: value_type.to_owned(),
            base_path: base_path.to_owned(),
            data: hash_map,
        }
    }

    #[must_use]
    pub fn metadata(&self) -> &str {
        &self.value_type
    }

    pub fn load(base_path: &Path) -> std::io::Result<Self> {
        let value_type = get_single_folder(&typeof_path(base_path))?;
        let mut data = HashMap::new();
        for entry in read_dir(data_path(base_path))? {
            let dir_entry = entry?;
            let key = dir_entry
                .file_name()
                .into_string()
                .map_err(|_| io::Error::from(ErrorKind::InvalidFilename))?;
            let value = get_single_folder(&dir_entry.path())?;
            data.insert(key, value);
        }
        Ok(Self {
            value_type,
            base_path: base_path.to_path_buf(),
            data,
        })
    }

    fn write_metadata(&self) -> std::io::Result<()> {
        let typeof_path = typeof_path(&self.base_path);
        fs::create_dir_all(typeof_path.join(self.value_type.as_str()))?;
        Ok(())
    }

    fn write_entries(&self) -> std::io::Result<()> {
        fs::create_dir_all(data_path(&self.base_path))?;
        for entries in &self.data {
            fs::create_dir_all(data_path(&self.base_path).join(entries.0).join(entries.1))?;
        }
        Ok(())
    }

    pub fn flush(&self) -> std::io::Result<()> {
        self.write_metadata()?;
        self.write_entries()?;
        Ok(())
    }

    pub fn insert(&mut self, k: String, v: String) -> std::io::Result<()> {
        fs::create_dir_all(data_path(&self.base_path).join(&k).join(&v))?;
        self.data.insert(k, v);
        Ok(())
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.data.get(k)
    }

    pub fn remove(&mut self, k: &String) -> std::io::Result<()> {
        fs::remove_dir_all(data_path(&self.base_path).join(k))?;
        self.data.remove(k);
        Ok(())
    }
}

fn get_single_folder(base_path: &Path) -> std::io::Result<String> {
    let mut entries = read_dir(base_path)?;

    let first_entry = entries
        .next()
        .ok_or_else(|| io::Error::from(ErrorKind::NotFound))??;

    first_entry
        .file_name()
        .into_string()
        .map_err(|_| io::Error::from(ErrorKind::InvalidFilename))
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;

    #[test]
    fn string_table() {
        let base_path = &temp_dir().join("string_table");
        let mut db: InMemoryTable = InMemoryTable::new("string", base_path);
        db.flush().unwrap();
        db.insert("foo".to_owned(), "bar".to_owned()).unwrap();
        db.insert("baz".to_owned(), "123".to_owned()).unwrap();

        let db2 = InMemoryTable::load(base_path).unwrap();
        assert_eq!(db, db2);
        db.remove(&"foo".to_owned()).unwrap();
        let db2 = InMemoryTable::load(base_path).unwrap();
        assert_eq!(db, db2);
    }
    #[test]
    fn number_table() {
        let base_path = &temp_dir().join("number_table");
        let mut db: InMemoryTable = InMemoryTable::new("number", base_path);
        db.flush().unwrap();
        db.insert("foo".to_owned(), "456".to_owned()).unwrap();
        db.insert("baz".to_owned(), "123".to_owned()).unwrap();

        let db2 = InMemoryTable::load(base_path).unwrap();
        assert_eq!(db, db2);
    }
}
