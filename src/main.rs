extern crate chrono;
extern crate clap;
extern crate ctrlc;
extern crate termion;

use chrono::DateTime;
use chrono::Duration;
use chrono::Local;

use clap::App;
use clap::Arg;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::thread;
use std::thread::park_timeout;

#[derive(Clone, Debug)]
struct State<'a> {
    width: u16,
    height: u16,
    start: DateTime<Local>,
    end: DateTime<Local>,
    task: Option<&'a str>,
    duration: Duration,
    remaining: Duration
}

enum Event {
    WindowSizeChange(u16, u16),
    CtrlC,
    Timeout(DateTime<Local>)
}

fn main() {
    let matches = App::new("Romodoro")
                        .version("0.4")
                        .about("Romodoro is a terminal Pomodoro timer.")
                        .author("Michael Fletcher <m.fletcher@theplanet.ca>")
                        .arg(Arg::with_name("task")
                               .long("task")
                               .help("Display the specified task on the timer.  This will help keep you focused.")
                               .takes_value(true))
                        .arg(Arg::with_name("duration")
                               .long("duration")
                               .help("Specify the duration of the pomodoro.  Defaults to 25m.")
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


    let mut last_width = 0;
    let mut last_height = 0;

    let (width, height) = termion::terminal_size().unwrap_or((80, 24));

    let mut state = match initialize_state(width, height, matches.value_of("task"), matches.value_of("duration")) {
        Ok(x) => x,
        Err(error_msg) => {
            println!("{}", error_msg);

            return;
        }
    };
        
    // Update the screen.
    while Local::now() < state.end && keep_running.load(Ordering::SeqCst) {
        let (width, height) = termion::terminal_size().unwrap_or((80, 24));

        if width != last_width || height != last_height {
            handle_event(Event::WindowSizeChange(width, height), &mut state);
            last_width = width;
            last_height = height;
        }

        handle_event(Event::Timeout(Local::now()), &mut state);
        
        if !keep_running.load(Ordering::SeqCst) {
            handle_event(Event::CtrlC, &mut state);
        }

        if state.remaining.num_seconds() > 120 { 
            // Update less frequently if we have a ways to go.
            park_timeout(std::time::Duration::from_secs(10));
        } else {
            park_timeout(std::time::Duration::from_secs(1));            
        }
    }

}

fn initialize_state<'a>(width:u16, height:u16, task:Option<&'a str>, duration:Option<&'a str>) -> Result<State<'a>, String> {
    let start = Local::now();
    let dur = match duration {
        Some(duration_str) => {
            let human_duration = humantime::parse_duration(duration_str);
            if human_duration.is_err() {
                return Err(format!("'{}' is not a valid duration.", duration_str));
            }

            Duration::from_std(human_duration.unwrap()).unwrap()
        },
        None => Duration::seconds(60 * 25)
    };

    Ok(State {
        start,
        end: start + dur,
        task,
        duration: dur,
        width,
        height,
        remaining: (start + dur) - start
    })
}

#[cfg(test)]
#[test]
fn initialize_state_tests() {
    let state = initialize_state(20, 80, None, Some("15m"));
    assert_eq!(true, state.is_ok());

    let state = initialize_state(20, 80, None, Some("15"));
    assert_eq!(false, state.is_ok());
    assert_eq!("'15' is not a valid duration.", state.unwrap_err());
}

fn draw_screen_reset() {
    // Revert the cursor, colours and style.
    println!("{}", termion::cursor::Show);
    println!("{}", termion::color::Fg(termion::color::Reset));
    println!("{}", termion::style::Reset);
} 

fn handle_event(event: Event, state:&mut State) {
    match event {
        Event::WindowSizeChange(w, h) => {
            state.width = w;
            state.height = h;

            draw_all(state);
        },
        Event::CtrlC => { 
            draw_screen_reset();
            std::process::exit(0);
        },
        Event::Timeout(now) => {
            if now > state.end {
                draw_screen_reset();
                std::process::exit(0);
            } 
            state.remaining = state.end - now;
        }
    };

    draw_changes(state);
}

fn draw_all(state:&State) {
    
    println!("{}", termion::clear::All);
    
    // Print Task
    if state.task.is_some() {
        let task_str = &state.task.unwrap();
        println!("{}{}{}{}{}{}", 
            termion::style::Bold,
            termion::color::Fg(termion::color::LightRed),
            termion::cursor::Goto((state.width / 2) - (task_str.len() / 2) as u16, state.height / 2), 
            &state.task.unwrap(),
            termion::color::Fg(termion::color::Reset),
            termion::style::Reset);
    }
    
    // Print Start
    println!("{}{}Start: {}{}", 
        termion::cursor::Goto(4, 2),
        termion::color::Fg(termion::color::Reset),
        termion::color::Fg(termion::color::LightBlue),
        state.start.format("%l:%M"),
        );

    // Print Duration
    let duration_str_for_size = format!("Duration: {}m", state.duration.num_minutes());
    let duration_str = format!(
        "{}Duration: {}{}m",
        termion::color::Fg(termion::color::Reset),
        termion::color::Fg(termion::color::LightBlue),
        state.duration.num_minutes()
        );
        
    println!("{}{}", 
        termion::cursor::Goto((state.width / 2) - (duration_str_for_size.len() / 2) as u16, 2),
        duration_str);

    // Print End
    let end_str = format!(
        "{}End: {}{}",
        termion::color::Fg(termion::color::Reset),
        termion::color::Fg(termion::color::LightBlue),
        state.end.format("%l:%M")
        );
    
    println!("{}{}", 
        termion::cursor::Goto(state.width - 9 - 4 as u16, 2),
        end_str);
    
    println!("{}", termion::cursor::Hide);
}

/**
 * TODO Take the clearing out, something else should be responsible.
 */
fn write_duration(dur: chrono::Duration, writer: &mut dyn std::io::Write) {
    if dur.num_seconds() > 60 {
        write!(writer, "{}m  ", dur.num_minutes() + 1).unwrap();
    } else {
        write!(writer, "{}s  ", dur.num_seconds()).unwrap();
    }
}


#[cfg(test)]
#[test]
fn duration_str_tests() {
    let make_str = |dur| {
        let mut buf:Vec<u8> = Vec::new();
        write_duration(dur, &mut buf);

        String::from_utf8(buf).unwrap()
    };

    assert_eq!("0s  ", make_str(Duration::zero()));
    assert_eq!("3s  ", make_str(Duration::seconds(3)));
    assert_eq!("60s  ", make_str(Duration::seconds(60)));
    assert_eq!("2m  ", make_str(Duration::seconds(61)));
}

fn num_bar_fill(remaining:chrono::Duration, _duration:chrono::Duration, bar_size:u16) -> u16 {
    let percent = 1.0 - (remaining.num_seconds() as f64 / _duration.num_seconds() as f64);
    
    (percent * f64::from(bar_size)) as u16
}


#[cfg(test)]
#[test]
fn num_bar_fill_tests() {
    assert_eq!(40, num_bar_fill(Duration::seconds(30), Duration::seconds(60), 80));    
}

fn draw_changes(state: &State) {
    print!("{}{}Remaining: {}", 
        termion::cursor::Goto(4, state.height - 3),
        termion::color::Fg(termion::color::Reset),
        termion::color::Fg(termion::color::LightBlue));
    write_duration(state.remaining, &mut std::io::stdout().lock());

    //    duration_str(remaining));

    let bar_size = state.width - 4 - 4;
    let progress_current = num_bar_fill(state.remaining, state.duration, state.width);

     // TODO pull out the string / percent 
    print!("{}", termion::color::Bg(termion::color::Blue));
    for c in 4..(4+progress_current) {
        print!("{} ", termion::cursor::Goto(c, state.height - 1))
    }
    print!("{}", termion::color::Bg(termion::color::Reset));

    print!("{}", termion::color::Bg(termion::color::White));
    for c in (4 + progress_current)..(bar_size+4) {
        print!("{} ", termion::cursor::Goto(c, state.height - 1))
    }

    println!("{}", termion::color::Bg(termion::color::Reset));
}