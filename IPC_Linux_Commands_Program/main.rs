use std::process::Command; // Spawns & manages external processes
use std::io::{self, Write, BufRead}; // Read process Output line by line
use std::time::Instant;// Used to track performance

// CS 3502 - Section 01 - jjncharl

// This executable will allow users to input linux commands and will execute them
fn main(){
    println!("Hello! Welcome to Linux Command Center. \n--------------------");
    loop{
        print!("Please enter a linux command and its argument(s) (or \"0\" to quit the program)
        \n [Example Input: cat somefile.txt]: ");
        //User input prompt
        io::stdout().flush().unwrap(); // Make sure the user-input prompt appears immediately.

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Sorry, input read has failed.");
        let input = input.trim(); //Remove newline chars

        if input == "0" {
            println!("Quitting Program. . .Goodbye!");
            break; //Exit the loop
        }

        let mut parts = input.split_whitespace(); // Split input into command and arg(s) by the whitespace
        if let Some(command) = parts.next() {
            let args: Vec<&str> = parts.collect(); // Collect the remaining arg(s) in an array

            let start = Instant::now(); // Create & start timer for Performance Benchmarking

            // Perform user-inputted command on given argument.
            let output = Command::new(command)
                .args(&args) // Pass arguments to the command!
                .output();

            let duration = start.elapsed(); // Stop timer

            match output { // A "switch" statement to choose how to handle user's chosen command
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                    println!("!! - Command executed in {:.6} seconds!\n", duration.as_secs_f64());
                }
                Err(e) => eprintln!("!! - There was an error executing the command. Reason: {}", e), //Prints errors to standard error; e contains error msg for why the command failed.

            }
        }
    }
}

// IPC validation testing.
mod tests {
    use super::*; // Imports all from the file so that it is in this scope
    #[test]
    fn test_ls_output_integrity() {
        let output = Command::new("ls")
            .output()
            .expect("!! - There was an error executing ls command.");

        let expected_files = vec!["lib.rs", "main.rs"]; // Arr of expected output files; change according to your load
        let output_str = String::from_utf8_lossy(&output.stdout); // Store ls output as String
        let files_found: Vec<&str> = output_str.split_whitespace().collect(); // Now, normalize ls output to check against assert! cases.

        for file in expected_files {
            assert!(files_found.contains(&file), "!! - File \"{}\" not found in ls output! Files found: {:?}", file, files_found);
        }
    }
}