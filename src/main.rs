use core::fmt;
use std::fs::File;
use std::iter;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use chrono::{DateTime, Utc};
use chrono::TimeZone;
use std::io::prelude::*;
use std::io::{self};

#[derive(Serialize, Deserialize, Debug)]
enum TodoListPriority {
    Low,
    Medium,
    High,
}

impl fmt::Display for TodoListPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
enum TodoListItemStatus {
    NotStarted,
    InProgress,
    Complete,
}

impl fmt::Display for TodoListItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoListItem {
    item_name: String,
    priority: TodoListPriority,
    due_date: DateTime<Utc>,
    status: TodoListItemStatus,
}

fn write_data_to_file(data: &Vec<TodoListItem>, file: &mut File) -> Result<()> {
    file.rewind().expect("Couldn't set cursor to start of file");
    file.set_len(0).expect("Couldn't clear file.");
    let serialized = serde_json::to_string_pretty(&data).expect("COuld not serialize json");
    file.write_all(serialized.as_bytes()).unwrap();
    Ok(())
}

fn read_data_from_file(file: &mut File) -> Result<Vec<TodoListItem>> {
    file.rewind().expect("Could not set cursor to start of file.");
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).expect("Failed to read file contents");
    let todo_list_items = serde_json::from_str::<Vec<TodoListItem>>(&file_contents.to_string()).expect("Failed to parse JSON");
    Ok(todo_list_items)
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
            file.write(b"[]").expect("Failed to write to file.");
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



    // // temp for writing to json file
    // let testing_data = vec![
    //     TodoListItem {
    //         item_name: String::from("test"),
    //         priority: TodoListPriority::Low,
    //         due_date: Utc.with_ymd_and_hms(2027, 1, 1, 12, 0, 0).unwrap(),
    //         status: TodoListItemStatus::NotStarted,
    //     },
    //     TodoListItem {
    //         item_name: String::from("test2"),
    //         priority: TodoListPriority::High,
    //         due_date: Utc.with_ymd_and_hms(2028, 1, 1, 12, 0, 0).unwrap(),
    //         status: TodoListItemStatus::InProgress,
    //     }
    // ];
    // write_data_to_file(&testing_data, &mut file).expect("Could not write data to file");
    // // end temp for writing to json file


    if file.metadata().unwrap().len() == 0 {
        file.write(b"[]").expect("Failed to write to file.");
    }

    let mut todo_list_items = read_data_from_file(&mut file).expect("Unable to read from file");

    for (i, item) in todo_list_items.iter().enumerate() {
        println!("{}. {} [{}] (Priority: {}) | Due: {}", i+1, item.item_name, item.status, item.priority, item.due_date.format("%d/%m/%Y %H:%M"));
    }

    
    // Show options of what user can do
    print!("\nType the number of a todo list item to modify it or a new todo list item name:");
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if let Ok(i) = input.parse::<usize>() {
            if i >= 1 && i <= todo_list_items.len() { // number selected
                let item = &mut todo_list_items[i-1];

                // Display item info
                println!("1. {}\n2. {}\n3. {}\n4. {}\n", item.item_name, item.status, item.priority, item.due_date);

                print!("Enter the desired line number to modify: ");
                loop {
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read user input");
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }
                    if let Ok(i) = input.parse::<usize>() {
                        if i == 1 {}
                    }
                }


                item.status = TodoListItemStatus::Complete;
                write_data_to_file(&todo_list_items, &mut file).unwrap();
                break
            }
            continue
        } else { // create new item
            // ask user other info

            
            let item = TodoListItem {
                item_name: input.to_string(),
                priority: TodoListPriority::Low,
                due_date: Utc::now() + chrono::Duration::days(1),
                status: TodoListItemStatus::NotStarted,
            };
            todo_list_items.push(item);
            write_data_to_file(&todo_list_items, &mut file).unwrap();
            break
        }
    }

    Ok(())
}
