use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use chrono::{DateTime, Utc};
use chrono::TimeZone;
use std::io::prelude::*;
use std::io::{self};

#[derive(Serialize, Deserialize, Debug)]
enum TodoListItemStatus {
    NotStarted,
    InProgress,
    Complete,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoListItem {
    item_name: String,
    priority: u8,
    due_date: DateTime<Utc>,
    status: TodoListItemStatus,
}

fn main() -> Result<()> {
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
    let mut file = match File::options()
    .read(true)
    .write(true)
    .open(&path) {
        Err(e) => panic!("Error opening {}: {}", display, e),
        Ok(file) => file,
    };



    // temp for writing to json file
    let testing_data = vec![
        TodoListItem {
            item_name: String::from("test"),
            priority: 3,
            due_date: Utc.with_ymd_and_hms(2027, 1, 1, 12, 0, 0).unwrap(),
            status: TodoListItemStatus::NotStarted,
        },
        TodoListItem {
            item_name: String::from("test2"),
            priority: 3,
            due_date: Utc.with_ymd_and_hms(2028, 1, 1, 12, 0, 0).unwrap(),
            status: TodoListItemStatus::InProgress,
        }
    ];

    let serialized = serde_json::to_string_pretty(&testing_data)?;
    file.write_all(serialized.as_bytes()).unwrap();

    // end temp for writing to json file




    file.rewind().expect("Couldn't set cursor to beginnging of file");
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).expect("Failed to read file contents");
    match serde_json::from_str::<Vec<TodoListItem>>(&file_contents.to_string()) {
        Ok(i) => println!("Successfully parsed todo list item: {:?}", i),
        Err(e) => println!("Failed to parse JSON: {}", e),
    }

    

    Ok(())
}
