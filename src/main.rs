use std::{path::PathBuf, fs, thread::{JoinHandle, self}, sync::{atomic::AtomicI32, Arc, Mutex}, process::{Stdio, Command}, time::{Instant, Duration}, str::from_utf8};

use clap::{Arg, ArgAction};
use process_control::{ChildExt, Control};
use colored::*;

#[derive(Debug, Clone)]
struct Test {
    name: String,
    input: PathBuf, // input file
    output: PathBuf // expected output file
}

fn split_in_packs<T>(original: Vec<T>, max_n: usize) -> Vec<Vec<T>> where T: Clone {
    // Calculate max number of items in pack
    let m = ((original.len() as f32) / (max_n as f32)).ceil() as usize;

    // Create packs
    let chunks =  original.chunks(m);
    let mut r: Vec<Vec<T>> = Vec::new();

    for chunk in chunks {
        r.push(chunk.to_vec());
    }

    return r;
}

fn pad_right(text: String, width: usize) -> String {
    let mut text = text.clone();
    text.push_str(" ".repeat(width - std::cmp::min(text.len(), width)).as_str());
    return text;
}

fn print_spacer() {
    let term_width = termion::terminal_size().unwrap().0;
    println!("{}", "=".repeat(term_width as usize));
}

fn main() {
    // Get command line arguments
    let m = clap::Command::new("Sprawdzarka v2")
        .arg(Arg::new("executable").action(ArgAction::Set).required(true))
        .arg(Arg::new("inputs").action(ArgAction::Set).default_value("./in"))
        .arg(Arg::new("outputs").action(ArgAction::Set).default_value("./out"))
        .arg(Arg::new("time_limit").action(ArgAction::Set).short('t').alias("tl").default_value("32"))
        .arg(Arg::new("mem_limit").action(ArgAction::Set).short('m').alias("ml").default_value("256"))
        .arg(Arg::new("max_threads").action(ArgAction::Set).short('l').alias("th").default_value("10"))
        .arg(Arg::new("print_failed").action(ArgAction::SetTrue).short('p').help("Print list of failed tests at the end"))
        .get_matches();

    let executable = m.get_one::<String>("executable").unwrap().clone();
    let inputs = m.get_one::<String>("inputs").unwrap().clone();
    let outputs = m.get_one::<String>("outputs").unwrap().clone();
    let time_limit = Arc::new(m.get_one::<String>("time_limit").unwrap().parse::<u64>().unwrap());
    let mem_limit = Arc::new(m.get_one::<String>("mem_limit").unwrap().parse::<usize>().unwrap() * 1_000_000);
    let thread_limit = m.get_one::<String>("max_threads").unwrap().parse::<usize>().unwrap();
    let print_failed = m.get_flag("print_failed");

    // Load tests
    let mut tests: Vec<Test> = Vec::new();
    for file in fs::read_dir(inputs).unwrap() {
        let file = file.unwrap().path();
        if file.extension().is_none() || file.extension().unwrap() != "in" { continue; } // skip outputs and other files

        // Get input
        let input_stem = file.file_stem().unwrap();

        // Find matching test output
        let mut output_path = PathBuf::new();
        output_path.push(outputs.clone());
        output_path.push(input_stem);
        let output_path = output_path.with_extension("out");

        // Check if output exists
        if !output_path.exists() {
            panic!("Could not find matching output for file {}", file.file_name().unwrap().to_str().unwrap());
        }

        // Add test to the list
        tests.push(Test {
            name: input_stem.to_str().unwrap().to_string(),
            input: file,
            output: output_path
        })
    }
    
    // Split tests into packs
    let test_count = tests.len();
    let tests = split_in_packs(tests, thread_limit);

    // Create counters
    let ok_counter = Arc::new(AtomicI32::new(0));
    let err_counter = Arc::new(AtomicI32::new(0));
    let tle_counter = Arc::new(AtomicI32::new(0));

    // Create list of failed tests
    let failed_tests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Run tests
    print_spacer();
    println!("Threads: {}", tests.len());
    println!("Test count: {}", test_count);

    print_spacer();
    let mut handles: Vec<JoinHandle<()>> = Vec::new(); // Vector to store thread handles
    let total_start = Instant::now();
    for chunk in tests {
        // Create clones so that there is no problem with borrowing
        let executable = executable.clone();
        let time_limit = Arc::clone(&time_limit);
        let mem_limit = Arc::clone(&mem_limit);


        let failed_tests = Arc::clone(&failed_tests);
        
        // And also clone counters
        let ok_counter = Arc::clone(&ok_counter);
        let err_counter = Arc::clone(&err_counter);
        let tle_counter = Arc::clone(&tle_counter);
    
        // Run code
        let handle = thread::spawn(move || {
            for test in chunk {
                let pretty_name = format!("[{}]", test.name);
                let pretty_name = pad_right(pretty_name, 10);
                // Start time measurement
                let start_time = Instant::now();

                // Create test command
                // println!("{} < {}", executable, test.input.to_str().unwrap());

                let runner = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} < {}", executable, test.input.to_str().unwrap()))
                    .stdout(Stdio::piped())
                    .spawn().expect("spawn child");

                // Get output and set limits
                let output_data = runner.controlled_with_output()
                    .time_limit(Duration::from_secs(*time_limit)).terminate_for_timeout()
                    .memory_limit(*mem_limit)
                    .wait();

                // Measure time of execution
                let exec_time = start_time.elapsed();

                // Unwrap output data
                let output_data = output_data.unwrap();

                // Check for TLE or OOM
                if output_data.is_none() {
                    println!("{} {} - {}s", pretty_name, "TLE".yellow().bold(), exec_time.as_secs());
                    tle_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    continue;
                }

                // Check result
                let output = output_data.unwrap().stdout;
                let output = from_utf8(output.as_slice()).unwrap().chars().filter(|c| !c.is_whitespace()).collect::<String>();
                let test_data = fs::read_to_string(test.output).unwrap().chars().filter(|c| !c.is_whitespace()).collect::<String>();
                if output == test_data {
                    println!("{} {} - {}s", pretty_name, "OK".green(), exec_time.as_secs_f32());
                    ok_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                } else {
                    println!("{} {}", pretty_name, "WRONG ANSWER".red().bold());
                    err_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Add this test to the list of failed ones
                    {
                        let mut failed_list = failed_tests.lock().unwrap();
                        let name = test.name.clone();
                        failed_list.push(name);
                    }
                }
            }     
        });
        handles.push(handle);
    }

    // Join all handles
    for handle in handles {
        handle.join().unwrap();
    }

    // Summary
    print_spacer();
    println!("OK - {} | TLE - {} | FAILED - {}", 
        ok_counter.load(std::sync::atomic::Ordering::Relaxed).to_string().green(),
        tle_counter.load(std::sync::atomic::Ordering::Relaxed).to_string().yellow(),
        err_counter.load(std::sync::atomic::Ordering::Relaxed).to_string().red(),
    );
    println!("Total testing time - {:.2}s", total_start.elapsed().as_secs_f32());

    if print_failed {
        print_spacer();
        let failed_data = failed_tests.lock().unwrap();
        println!("{} {}", "Failed tests:".red(), failed_data.join(", "));
    }

}
