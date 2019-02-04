extern crate chrono;
extern crate clap;
extern crate ctrlc;
extern crate termion;

use chrono::Duration;
use chrono::Local;

use clap::App;
use clap::Arg;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::thread;
use std::thread::park_timeout;

fn main() {
    let matches = App::new("Romodoro")
                        .version("0.3")
                        .about("Romodoro is a terminal Pomodoro timer.")
                        .author("Michael Fletcher <m.fletcher@theplanet.ca>")
                        .arg(Arg::with_name("task")
                               .long("task")
                               .help("Display the specified task on the timer.  This will help keep you focused.")
                               .takes_value(true))
                        .get_matches();

    // Setup a CTRL-C handler so we can cleanly close.  This is basically ensuring we reset the colours and cursor.
    let keep_running_in_handler = Arc::new(AtomicBool::new(true));
    let keep_running = keep_running_in_handler.clone();

    let current_thread = thread::current();
    ctrlc::set_handler(move || {
        keep_running_in_handler.store(false, Ordering::SeqCst);
        current_thread.unpark();
    }).unwrap();

    let start = Local::now();
    let duration = Duration::seconds(60 * 25);
    let end = start + duration;    

    let task = matches.value_of("task");
    let total_seconds = (end - start).num_seconds();

    let mut last_width = 0;
    let mut last_height = 0;

    // Update the screen.
    while Local::now() < end && keep_running.load(Ordering::SeqCst) {
        let (width, height) = termion::terminal_size().unwrap_or((80, 24));
    
        if width != last_width || height != last_height {
            last_width = width;
            last_height = height;

            println!("{}", termion::clear::All);
            
            // Print Task
            if task.is_some() {
                let task_str = &task.unwrap();
                println!("{}{}{}{}{}{}", 
                    termion::style::Bold,
                    termion::color::Fg(termion::color::LightRed),
                    termion::cursor::Goto((width / 2) - (task_str.len() / 2) as u16, height / 2), 
                    &task.unwrap(),
                    termion::color::Fg(termion::color::Reset),
                    termion::style::Reset);
            }
            
            // Print Start
            println!("{}{}Start: {}{}", 
                termion::cursor::Goto(4, 2),
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::LightBlue),
                start.format("%l:%M"),
                );

            // Print Duration
            let duration_str_for_size = format!("Duration: {}m", duration.num_minutes());
            let duration_str = format!(
                "{}Duration: {}{}m",
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::LightBlue),
                duration.num_minutes()
                );
                
            println!("{}{}", 
                termion::cursor::Goto((width / 2) - (duration_str_for_size.len() / 2) as u16, 2),
                duration_str);

            // Print End
            let end_str = format!(
                "{}End: {}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::LightBlue),
                end.format("%l:%M")
                );
            
            println!("{}{}", 
                termion::cursor::Goto(width - 9 - 4 as u16, 2),
                end_str);
            
            println!("{}", termion::cursor::Hide);
        }

        let remaining = end - Local::now();

        if remaining.num_seconds() > 60 { 
            // two extra spaces to cover when it changes from 10m to 9m.  Without the extra space
            // it shows "9mm"
            println!("{}{}Remaining: {}{}m  ", 
                termion::cursor::Goto(4, height - 3),
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::LightBlue),
                remaining.num_minutes() + 1);;
        } else {
            // See spaces comment above.
            println!("{}{}Remaining: {}{}s  ", 
                termion::cursor::Goto(4, height - 3),
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::LightBlue),
                remaining.num_seconds());;
        }
        
        let percent = 1.0 - (remaining.num_seconds() as f64 / total_seconds as f64);
        let progress_max = width - 4 - 4;
        let progress_current = (percent * f64::from(progress_max)) as u16;

        print!("{}", termion::color::Bg(termion::color::Blue));
        for c in 4..(4+progress_current) {
            print!("{} ", termion::cursor::Goto(c, height - 1))
        }
        print!("{}", termion::color::Bg(termion::color::Reset));

        print!("{}", termion::color::Bg(termion::color::White));
        for c in (4 + progress_current)..(progress_max+4) {
            print!("{} ", termion::cursor::Goto(c, height - 1))
        }

        println!("{}", termion::color::Bg(termion::color::Reset));

        if remaining.num_seconds() > 120 { 
            // Update less frequently if we have a ways to go.
            park_timeout(std::time::Duration::from_secs(10));
        } else {
            park_timeout(std::time::Duration::from_secs(1));            
        }
    }

    // Revert the cursor, colours and style.
    println!("{}", termion::cursor::Show);
    println!("{}", termion::color::Fg(termion::color::Reset));
    println!("{}", termion::style::Reset);
    
    

}