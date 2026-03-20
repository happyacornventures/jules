use serde_json::{json, Value};
use std::fs::{self, create_dir_all, File};
use std::io::{Read, Result};

pub fn write_file(file_name: &str, content: &Value) -> Result<()> {
    let data = json!(content);
    let path = std::path::Path::new(file_name);
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    File::create(file_name)?;
    fs::write(file_name, data.to_string())?;
    Ok(())
}

pub fn read_file(file_name: &str, default_value: Value) -> Result<String> {
    let mut buffer = String::new();
    let mut file = match File::open(file_name) {
        Ok(file) => file,
        Err(_) => {
            write_file(file_name, &default_value)?;
            File::open(file_name)?
        }
    };

    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn list_directory(path: String) -> Result<Vec<String>> {
    let mut entries = fs::read_dir(path.clone())?;
    let mut files: Vec<String> = Vec::new();
    while let Some(entry) = entries.next() {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            files.push(file_name);
        } else if entry.file_type()?.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            files.extend(list_directory(format!("{}/{}", path.clone(), dir_name))?);
            files.push(dir_name);
        }
    }
    Ok(files)
}
