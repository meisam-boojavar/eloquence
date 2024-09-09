use std::{fs, io};
use std::fs::canonicalize;
use postgres::{Client, NoTls, Error};
use std::path::Path;
use regex::Regex;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn check() -> bool {

    let current_dir = std::env::current_dir().unwrap();
    let target_dir = current_dir.join("migrations");
    let _ = target_dir.is_dir();

    let filenames = list("migrations");

    match filenames {
        Ok(filenames) => {

            for filename in filenames {

                println!("{}", filename)
            }
        }
        Err(e) => {

            println!("{}", e)
        }
    }

    return true;
}

pub fn makeconnection(connection_str: &str) -> Result<Client, Error> {

    Client::connect(connection_str, NoTls)
}

pub fn init(mut connection: Client) -> Result<(), std::io::Error> {

    let dir_path = Path::new("migrations");

    if dir_path.is_dir() {

        for entry in fs::read_dir(dir_path)? {

            let entry = entry?;
            let path = entry.path();

            if path.is_file() {

                let content = fs::read_to_string(path)?;

                let result = parsetovector(content.as_str());

                let sql = make_alter_table_sql(result);

                println!("{:?}", entry.file_name());

                if sql.clone() != "" {

                    match connection.batch_execute(&sql) {

                        Ok(_) => println!("SQL executed successfully!"),

                        Err(e) => { println!("{}", e.to_string())},
                    };
                } else {

                    println!("no correct migration content found./");
                }

            }
        }
    }

    Ok(())
}

pub fn list(dir_name: &str) -> io::Result<Vec<String>> {

    let current_dir = std::env::current_dir()?;
    let target_dir = current_dir.join(dir_name);

    if target_dir.is_dir() {

        let mut file_names = Vec::new();

        for entry in fs::read_dir(target_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {

                if let Some(file_name) = path.file_name() {

                    if let Some(file_name_str) = file_name.to_str() {
                        file_names.push(file_name_str.to_string());
                    }
                }
            }
        }
        Ok(file_names)
    }
    else {

        Err(io::Error::new(io::ErrorKind::NotFound, "Directory not found"))
    }
}

pub fn inside(dir_name: &str) -> bool {

    let current_dir = std::env::current_dir().unwrap();
    let target_dir = current_dir.join(dir_name);
    target_dir.is_dir()
}

pub fn get(model: &str, resolution: &str, condition: &str) {

    let h = transform_string(condition);

    println!("{}", h);
    // let clauses  = extract_clause(condition);
    //
    // println!("{:?}", clauses);
    //
    // for (index, clause) in clauses.iter().enumerate() {
    //
    //     if !clause.contains(',') {
    //
    //         //println!("{:?}", parsetovector(clause));
    //     }
    //     if clause.contains(',') {
    //
    //         let mut and_clause = String::from("");
    //
    //         let items = parsetovector(clause);
    //
    //         for (index, value) in items.iter().enumerate() {
    //
    //             if index == (items.len() - 1) {
    //
    //                 and_clause = and_clause + value;
    //             } else {
    //
    //                 and_clause = and_clause + value + " AND ";
    //             }
    //         }
    //
    //         //println!("{:?}", and_clause);
    //         //println!("ot")
    //     }
    // }

    //println!("{:?}", extract_blobs(condition));
}

fn parsetovector(input: &str) -> Vec<String> {

    let mut result = input
        .replace("\n", "")
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim_matches(|c| c == '\'' || c == ' ').to_string())
        .collect::<Vec<String>>();

    result.retain(|s| !s.is_empty());

    result
}

fn extract_clause(input: &str) -> Vec<String> {

    let mut blobs = Vec::new();
    let mut stack = Vec::new();
    let mut current_blob = String::new();
    let mut in_blob = false;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '(' => {
                if stack.is_empty() {
                    in_blob = true; // We're at the start of a top-level blob
                }
                stack.push(i);
            }
            ')' => {
                if let Some(start) = stack.pop() {
                    if stack.is_empty() {
                        // We're closing a top-level blob
                        let blob = &input[start + 1..i];
                        if !blob.trim().is_empty() {
                            blobs.push(blob.trim().to_string());
                        }
                        in_blob = false;
                    }
                }
            }
            ',' => {
                if stack.len() == 1 && in_blob {
                    // We have a top-level delimiter
                    if !current_blob.trim().is_empty() {
                        blobs.push(current_blob.trim().to_string());
                        current_blob.clear();
                    }
                } else {
                    current_blob.push(chars[i]);
                }
            }
            _ => current_blob.push(chars[i]),
        }
        i += 1;
    }

    if !current_blob.trim().is_empty() {
        blobs.push(current_blob.trim().to_string());
    }

    blobs
}

fn transform_string(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    for ch in input.chars() {
        match ch {
            '(' => result.push_str("("), // Replace '(' with "remove"
            ',' => result.push_str(" AND "), // Replace ',' with " AND "
            ')' => {
                result.push_str(") OR "); // Replace ')' with "hasan"
                if chars.peek() == Some(&'(') {
                    chars.next(); // Consume the comma
                    result.push_str(" special"); // Replace ", " with " special"
                }
            }
            _ => result.push(ch), // Keep other characters as they are
        }
    }

    result
}

fn make_alter_table_sql(input: Vec<String>) -> String {

    let mut sql = String::from("");

    for (index, value) in input.iter().enumerate() {

        if index == 0 {

            sql = sql + "CREATE TABLE " + value + " (";
        }

        if index > 0 {

            if index == (input.len() -1)  {

                sql = sql + value
            }
            else {

                sql = sql + value + ", ";
            }
        }
    }

    if input.len() > 0 {

        sql = sql + ")";
    }

    sql
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
