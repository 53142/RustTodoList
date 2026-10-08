use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use chrono::{DateTime, Utc};
use chrono::TimeZone;
use std::io::prelude::*;
use std::io::{self};

#[derive(Serialize, Deserialize)]
enum TodoListItemStatus {
    NotStarted,
    InProgress,
    Complete,
}

#[derive(Serialize, Deserialize)]
struct TodoListItem {
    itemName: String,
    priority: u8,
    dueDate: DateTime<Utc>,
    status: TodoListItemStatus,
}

fn main() -> Result<()> {
    let testing_data = TodoListItem {
        itemName: String::from("test"),
        priority: 3,
        dueDate: Utc.with_ymd_and_hms(2027, 1, 1, 12, 0, 0).unwrap(),
        status: TodoListItemStatus::NotStarted,
    };

    let serialized = serde_json::to_string(&testing_data)?;
    println!("{}", serialized);

    println!("Searching for existing todo list file");
    let path = Path::new("TodoList.json");

    if !path.exists() {
        println!("Unable to find an existing todo list file. Do you want to create one?");

        let response = loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read user input");

            match input.trim().to_lowercase().as_str() {
                "yes" | "y" => break true,
                "no" | "n" => break false,
                _ => println!("Invalid input."),
            }
        };

        if response {
            // create and initialize file
            let mut file = File::create(&path).expect("Failed to create file");
            file.write(b"").expect("Failed to write to file.");
            println!("Successfully created file.");
        } else {
            println!("Exiting...\nYou must allow the todolist file to be created to use the program!");
            return Ok(())
        };
    } else {
        println!("Found todo list file!");
    }

    let display = path.display();
    let mut file = match File::open(&path) {
        Err(e) => panic!("Error opening {}: {}", display, e),
        Ok(file) => file,
    };
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).expect("couldn't read");
    Ok(())
}
