use console::Term;
use console::Key;
use std::thread;
use std::env;
use std::io::Write;

type BuiltinFunc = fn(Vec<String>) -> i8;

const LSH_RL_BUFSIZE: usize = 1024;
const LSH_TOK_SIZE: usize = 64;
const LSH_TOK_DELIM: &[char] = &[' ', '\t', '\r', '\n', '\x07'];
const BUILTIN_STR: &[&str] = &["cd", "help", "exit"];

fn lsh_num_builtins() -> usize {
    return BUILTIN_STR.len()
}

fn lsh_cd(args: Vec<String>) -> i8 {
    if args[1].is_empty() {
        println!("Error: expected argument to \"cd\"\n");
    } else {
        let result = env::set_current_dir(&args[1]);
        if result.is_err() {
            println!("Error: Could not find {}", args[1]);
        }
    }
    return 1
}

fn lsh_help(args: Vec<String>) -> i8{
    println!("Type program names and arguments, and hit enter. \n");
    println!("The following are built in: \n");
    for i in 1..args.len() {
        println!("{}\n", args[i]);
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

    if args[0].is_empty() {
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
                if status.success() { 0 } else { 1 }
            }
            Err(err) => {
                println!("Error: {}", err);
                -1
            }
        }
    });

    let result = handle.join().unwrap();
    return result;
}

fn lsh_split_line(line: String) -> Vec<String>{
    let mut tokens: Vec<String> = Vec::with_capacity(LSH_TOK_SIZE * size_of::<String>());
    let mut token: &str;

    let mut split_line = line.split(LSH_TOK_DELIM);
    token = split_line.next().unwrap();

    while !token.is_empty() {
        tokens.push(String::from(token));
        if split_line.next().is_some() { token = split_line.next().unwrap(); }
    }
    return tokens;
}

fn lsh_read_line() -> String {
    let mut buffer: Vec<char> = Vec::with_capacity(LSH_RL_BUFSIZE * size_of::<char>());

    // Don't need to look out for resizing or anything
    // Because pushing onto Vector does that automatically
    loop {
        let term = Term::stdout();
        let c = term.read_char().unwrap();
        match Key::Char(c) {
            Key::Enter => {
                println!();
                return buffer.into_iter().collect();
            }
            Key::Backspace => {
                if !buffer.is_empty() {
                    buffer.pop();
                    term.move_cursor_left(1).unwrap();
                    std::io::stdout().flush().unwrap();
                }
            }
            Key::Char(c) => {
                buffer.push(c);
                print!("{}", c);
                std::io::stdout().flush().unwrap();
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

        if status == 0 {
            break;
        }
    }
}

fn main() {
    lsh_loop();
}