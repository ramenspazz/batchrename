use clap::Parser;
use std::process::exit;

// TODO: enable help text for directivies like "no-type", "auto-detect", and "delete-match"

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    pathname: Option<String>,
    #[arg(short, long)]
    space_replace: Option<bool>,
    #[arg(short, long)]
    match_value: Option<String>,
    #[arg(short, long)]
    replace_value: Option<String>,
    #[arg(short, long)]
    clean_tags: Option<bool>,
    #[arg(short, long)]
    descriptor_type: Option<String>,
}

fn do_nothing() {}

fn return_descriptor_type(process_string: &String) -> Option<String> {
    let mut descriptor_slice: Option<String> = None;

    for (i, item) in process_string.chars().rev().enumerate() {
        if item == '.' {
            descriptor_slice = Some(String::from(
                &process_string[&process_string.len() - i - 1..process_string.len()]
            ));
            break
        }
    }

    return descriptor_slice;
}

fn main() {
    let args = Cli::parse();
    // println!("pattern: {:?}, path: {:?}", args.pattern, args.path);
    // the first argument will be the path if it is valid. Check it
    let working_path = args.pathname.unwrap_or(String::from("."));
    println!("Working directory = {}\n", &working_path);
    if 
    (args.clean_tags.is_none() && args.space_replace.is_none() && args.match_value.is_none())
    || 
    (args.match_value.is_some() && args.replace_value.is_none())
    ||
    (args.match_value.is_none() && args.replace_value.is_some()) {
        if args.clean_tags.is_none() {
            // if all inputs are none, then throw an error and exit
            println!("Need to have space replace turned on, or provide a pattern to match and replace! Try using the `-h` flag for help.");
            exit(1);
        }
    }

    match std::fs::read_dir(working_path) {
        Ok(directory_iterator) => {
            // Path is valid, search the files here
            
            'file_iteration: for file_result in directory_iterator {
                match file_result {
                    Ok(file_n) => {
                        // get the original filename
                        let original_fname = file_n
                            .file_name()
                            .into_string()
                            .unwrap();
                        let mut old_fname = String::from(&original_fname);
                        let mut modded_fname = String::from(&old_fname);
                        let mut new_fname = String::from("");

                        // determine the file descriptor
                        let descriptor_slice: Option<String> = match args.descriptor_type {
                            Some(ref d_string) => {
                                // save descriptor and remove from working file name
                                if d_string == "auto-detect" {
                                    match return_descriptor_type(&old_fname) {
                                        Some(value) => {
                                            old_fname = old_fname.replace(&value, "");
                                            modded_fname = String::from(&old_fname);
                                            Some(value)
                                        },
                                        None => {
                                            println!("No file type descriptor found. Setting descriptor as None.");
                                            None
                                        },
                                    }
                                }
                                else if d_string == "no-type" {
                                    // ignore any filetype and returns None as if no descriptor string was passed
                                    None
                                }
                                else {
                                    if !old_fname.contains(d_string) {
                                        println!("skipping \"{}\". Not of type \"{}\"", &old_fname, &d_string);
                                        continue 'file_iteration;
                                    }
                                    old_fname = old_fname.replace(d_string, "");
                                    // println!("descriptor removed: {}", old_fname);
                                    modded_fname = String::from(&old_fname);
                                    Some(String::from(d_string))
                                }
                            },
                            None => None,
                        };

                        // skip directories
                        if file_n.file_type().unwrap().is_dir() == true { continue 'file_iteration }

                        // replace spaces first
                        match args.space_replace {
                            Some(flag_value) => {
                                if flag_value == true {
                                    new_fname = old_fname
                                        .replace(" ", "_");
                                    modded_fname = String::from(&new_fname);
                                }
                                else {
                                    continue 'file_iteration;
                                };
                            },
                            // no argument for space replace provided even though flag was
                            None => do_nothing(),
                        } // match args.space_replace

                        // search and replace
                        match (&args.match_value, &args.replace_value) {
                            (Some(match_val), Some(replace_val)) => {
                                if replace_val == "delete-match" {
                                    modded_fname = modded_fname.replace(match_val, "");
                                }
                                modded_fname = modded_fname.replace(match_val, &replace_val);
                                new_fname = String::from(&modded_fname);
                            },
                            (None, None) => do_nothing(),
                            (Some(_), None) => do_nothing(),
                            (None, Some(_)) => do_nothing(),
                        }

                        // search the tags for matching pairs of () and [] to move to the end before the file type designator
                        match args.clean_tags {
                            Some(clean_bool) => {
                                if clean_bool == true {
                                    // clean
                                    // let mut parenthesis_count = 0;
                                    let mut bracket_count = 0;
                                    let temp_name = String::from(&modded_fname);
                                    
                                    let mut tag_slice_vector: Vec<&str> = vec![];
                                    let mut start_i = 0;
                                    for (i, item) in temp_name.chars().enumerate() {
                                        if item == '[' {
                                            bracket_count += 1;
                                            start_i = i;
                                        }
                                        else if item == ']' {
                                            if bracket_count % 2 == 1 {
                                                // add to bracket_pair
                                                bracket_count -= 1;
                                                tag_slice_vector.push(&temp_name[start_i..i+1]);
                                            }
                                            else {
                                                println!("Unmatched bracket pair! Exiting...");
                                                exit(1);
                                            }
                                        }
                                    }
                                    // update name
                                    if !tag_slice_vector.is_empty() {
                                        for tag in tag_slice_vector {
                                            modded_fname = modded_fname.replace(tag, "");
                                            modded_fname = [modded_fname, String::from(tag)].join("");
                                        }
                                        new_fname = String::from(&modded_fname);
                                    }
                                    else { println!("No tags found"); }
                                } // if clean_bool == true
                            },
                            None => do_nothing(),
                        } // match args.clean_tags

                        // rename the file, start by checking to see if the filename was actually changed
                        if new_fname == old_fname {
                            // no change to the file, skip
                            continue 'file_iteration;
                        }
                        // check that the file path is valid
                        match &file_n.path().into_os_string().into_string() {
                            Ok(path_str) => {  
                                // deal with the file descriptor
                                let new_path = match descriptor_slice {
                                    Some(ref d_string) => {
                                        // if there was a file descriptor type passed, add it back to the filename
                                        let temp_path = path_str.replace(d_string, "");
                                        [temp_path.replace(&old_fname, &new_fname), String::from(d_string)].join("")
                                    },
                                    // No descriptor passed, assume if it did exist it is another part of the fname
                                    None => {
                                        // println!("old = {}, new = {}", &old_fname, &new_fname);
                                        path_str.replace(&original_fname, &new_fname)
                                    },
                                };
                                // rename the file
                                match std::fs::rename(&path_str, &new_path) {
                                    Ok(_) => {
                                        // sucessfully renamed file, print message to log and continue
                                        println!("renamed to: {}", &new_path); // possibly change this to be only the filename
                                        continue 'file_iteration;
                                    },
                                    Err(e) => {
                                        println!("rename error for \"{}\": {:?}\nattempted rename: {}", &path_str, e, &new_fname);
                                        exit(1);
                                    },
                                }
                            },
                            Err(e1) => {
                                // couldnt rename file. print error and exit
                                println!("Couldnt rename file {:?}\npath error: {:?}", &old_fname, e1);
                                exit(1);
                            },

                        } // match &file_n.path().into_os_string().into_string() 
                    },
                    Err(_) => continue 'file_iteration,
                }
            }
        },
        Err(_) => {
            // Path is invalid, display an error message and abort the program
            println!("Error, invalid path! Exiting...");
            exit(1);
        },
    };
}
