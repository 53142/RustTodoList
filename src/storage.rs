use color_eyre::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::models::TodoListItem;

pub(crate) fn write_data_to_file(
    path: &Path,
    data: &[TodoListItem],
) -> Result<()> {
    let mut file = File::create(path)?;
    let serialized = serde_json::to_string_pretty(data)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub(crate) fn read_data_from_file(
    path: &Path,
) -> Result<Vec<TodoListItem>> {
    // Create file if doesn't already exist and return
    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(b"[]")?;

        return Ok(vec![]);
    }

    let mut file = File::open(path)?;

    if file.metadata()?.len() == 0 {
        return Ok(vec![]);
    }

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let items = serde_json::from_str(&file_contents)?;
    Ok(items)
}