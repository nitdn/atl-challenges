use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::{Path, PathBuf},
};
pub struct InMemoryDB {
    value_type: String,
    base_path: PathBuf,
    data: HashMap<String, String>,
}

impl std::ops::DerefMut for InMemoryDB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl std::ops::Deref for InMemoryDB {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

fn typeof_path(base_path: &Path) -> PathBuf {
    base_path.join("metadata").join("type")
}

impl InMemoryDB {
    #[must_use]
    pub fn new(value_type: &str, base_path: &Path) -> Self {
        let hash_map = HashMap::new();
        Self {
            value_type: value_type.to_owned(),
            base_path: base_path.to_owned(),
            data: hash_map,
        }
    }
    fn write_metadata(&self) -> Result<(), Box<dyn Error>> {
        let typeof_path = typeof_path(&self.base_path);
        fs::create_dir_all(typeof_path.join(self.value_type.as_str()))?;
        Ok(())
    }

    fn write_entries(&self) -> Result<(), Box<dyn Error>> {
        for entries in &self.data {
            fs::create_dir_all(self.base_path.join(entries.0).join(entries.1))?;
        }
        Ok(())
    }

    pub fn flush(&self) -> Result<(), Box<dyn Error>> {
        self.write_metadata()?;
        self.write_entries()?;
        Ok(())
    }
}

impl Drop for InMemoryDB {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;

    #[test]
    fn metadata() {
        let mut db: InMemoryDB = InMemoryDB::new("string", &temp_dir().join("result"));
        db.insert("foo".to_owned(), "bar".to_owned());
        db.insert("bar".to_owned(), "123".to_owned());

        db.flush().unwrap();
    }
}
