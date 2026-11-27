use std::sync::Mutex;
use console::Key;
use console::Term;
use std::thread;
use std::env;
use std::io::Write;
#[macro_use]
extern crate lazy_static;

type BuiltinFunc = fn(Vec<String>) -> i8;

struct History {
    hist: Vec<String>,
    current_pos: usize,
}

const LSH_RL_BUFSIZE: usize = 1024;
const LSH_TOK_SIZE: usize = 64;
const LSH_TOK_DELIM: &[char] = &[' ', '\t', '\r', '\n', '\x07'];
const BUILTIN_STR: &[&str] = &["cd", "help", "exit"];
lazy_static! {
    static ref HISTORY: Mutex<History> = {
        let h = History { hist: Vec::new(), current_pos: 0 };
        Mutex::new(h)
    };
}

fn lsh_fuzzy_search() {
    // Implement here;
}

fn lsh_num_builtins() -> usize {
    return BUILTIN_STR.len()
}

fn lsh_cd(args: Vec<String>) -> i8 {
    if !args.get(1).is_some() {
        println!("Error: expected argument to \"cd\"\n");
    } else {
        let result = env::set_current_dir(&args[1]);
        match result {
            Ok(()) => {
                println!("Moved to: {}", &args[1]);
            }
            Err(err) => {
                println!("Error: {}", err.to_string());
            }
        }
    }
    return 1
}

fn lsh_help(_args: Vec<String>) -> i8{
    println!("Type program names and arguments, and hit enter. \n");
    println!("The following are built in: \n");
    for i in 0..BUILTIN_STR.len() {
        println!("{}\n", BUILTIN_STR[i]);
    }
    println!("Use the man command for information on other programs");
    return 1
}

fn lsh_exit(_args: Vec<String>) -> i8{
    return 0
}

fn lsh_execute(args: Vec<String>) -> i8 {
    let builtin_func: [BuiltinFunc; 3] = [
        lsh_cd,
        lsh_help,
        lsh_exit,
    ];

    if args.is_empty() || args[0].is_empty() {
        return 1
    }
    for i in 0..lsh_num_builtins() {
        if args[0] == BUILTIN_STR[i] {
            return builtin_func[i](args)
        }
    }

    return lsh_launch(args)
}

fn lsh_launch(args: Vec<String>) -> i8{
    let handle = thread::spawn(move || {
        let status = std::process::Command::new(&args[0])
            .args(&args[1..])
            .status();
        match status {
            Ok(status) => {
                if status.success() { 1 } else { 0 }
            }
            Err(err) => {
                println!("Error: {}", err);
                -1
            }
        }
    });

    let _ = handle.join();
    return 1
}

fn lsh_split_line(line: String) -> Vec<String>{
    let mut tokens: Vec<String> = Vec::with_capacity(LSH_TOK_SIZE * size_of::<String>());
    let mut token: &str;

    let mut split_line = line.split(LSH_TOK_DELIM);
    token = split_line.next().unwrap();

    while !token.is_empty() {
        tokens.push(String::from(token));
        match split_line.next() {
            Some(t) => {
                token = t;
            }
            None => {
                token = "";
            }
        }
    }
    return tokens;
}

fn lsh_read_line() -> String {
    let mut buffer: Vec<char> = Vec::with_capacity(LSH_RL_BUFSIZE * size_of::<char>());

    // Don't need to look out for resizing or anything
    // Because pushing onto Vector does that automatically
    loop {
        let term = Term::stdout();
        // I guess let 0 be the newest entry and move from there
        match term.read_key().unwrap() {
            Key::Enter => {
                let mut hist = HISTORY.lock().unwrap();
                if hist.hist.len() >= 500 {
                    hist.hist.remove(0);
                }
                hist.hist.push(buffer.clone().into_iter().collect());
                term.write_line("").unwrap();
                return buffer.into_iter().collect();
            }
            Key::Backspace | Key::Del => {
                if buffer.pop().is_some() {
                    term.clear_chars(1).unwrap();
                    std::io::stdout().flush().unwrap();
                }
            }
            Key::Char(c) => {
                if c == '\t' {
                    // insert auto complete here or smth;
                }
                buffer.push(c);
                print!("{}", c);
                std::io::stdout().flush().unwrap();
            }
            Key::CtrlC => {
                term.write_line("").unwrap(); std::process::exit(1);
            }
            Key::ArrowUp => {
                let mut hist = HISTORY.lock().unwrap();
                if !hist.hist.is_empty() && !(hist.current_pos == hist.hist.len()) {
                    let _ = term.clear_line();
                    print!("> {}", hist.hist.get(hist.current_pos).unwrap());
                    std::io::stdout().flush().unwrap();
                    hist.current_pos += 1;
                }
            }
            Key::ArrowDown => {
                let mut hist = HISTORY.lock().unwrap();
                if !hist.hist.is_empty() && !(hist.current_pos <= 1) {
                    let _ = term.clear_line();
                    print!("> {}", hist.hist.get(hist.current_pos - 1).unwrap());
                    std::io::stdout().flush().unwrap();
                    hist.current_pos -= 1;
                }
                else {
                    hist.current_pos = 0;
                    let _ = term.clear_line();
                    print!("\r> ");
                    std::io::stdout().flush().unwrap();
                }
            }
            _ => {}
        }
    }
}

fn lsh_loop() {
    let mut status: i8;
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        status = lsh_execute(lsh_split_line(lsh_read_line()));

        if status <= 0 {
            break;
        }
    }
}

fn main() {
    lsh_loop();
}