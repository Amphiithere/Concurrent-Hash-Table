use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{env, fmt};

pub enum Command
{
    Insert { name: String, salary: u32, priority: u32 },
    Update { name: String, salary: u32, priority: u32 },
    Delete { name: String, priority: u32 },
    Search { name: String, priority: u32 },
    Print { priority: u32 }
}

impl Command
{
    pub fn name(&self) -> Option<String> {
        return match self {
            Command::Insert { name, .. } |
            Command::Update { name, .. } |
            Command::Delete { name, .. } |
            Command::Search { name, .. } => Some(name.clone()),
            _ => None
        };
    }

    pub fn name_ref(&self) -> Option<&String> {
        return match self {
            Command::Insert { name, .. } |
            Command::Update { name, .. } |
            Command::Delete { name, .. } |
            Command::Search { name, .. } => Some(name),
            _ => None
        }
    }

    pub fn salary(&self) -> Option<u32> {
        return match self {
            Command::Insert { salary, .. } |
            Command::Update { salary, .. } => Some(*salary),
            _ => None
        };
    }

    pub fn priority(&self) -> u32 {
        return match self {
            Command::Insert { priority, .. } |
            Command::Update { priority, .. } |
            Command::Delete { priority, .. } |
            Command::Search { priority, .. } |
            Command::Print { priority, .. } => *priority
        };
    }
}

impl fmt::Display for Command
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        return match self {
            Command::Insert { name, salary, priority } => {
                write!(f, "insert,{},{},{}", name, salary, priority)
            }
            Command::Update { name, salary, priority } => {
                write!(f, "update,{},{},{}", name, salary, priority)
            }
            Command::Delete { name, priority } => {
                write!(f, "delete,{},{}", name, priority)
            }
            Command::Search { name, priority } => {
                write!(f, "search,{},{}", name, priority)
            }
            Command::Print { priority } => {
                write!(f, "print,{}", priority)
            }
        };
    }
}



fn open_commands_file_with_buffered_reader() -> io::Result<io::Lines<io::BufReader<File>>>
{
    const COMMANDS_FILE: &str = "./commands.txt";
    let path = Path::new(COMMANDS_FILE);

    // Error Handling
    if !path.exists()
    {
        match env::current_dir()
        {
            Ok(cwd) => {
                eprintln!(
                    "Error: Command file '{}' not found in directory '{}'.",
                    COMMANDS_FILE,
                    cwd.display()
                );
            }
            Err(e) => eprintln!("Failed to resolve current working directory:\n{}", e),
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Command file '{}' missing", COMMANDS_FILE),
        ));
    }

    // Return BufReader to iterate lines
    // ? operator suffixing the function call makes it so that it
    // immediately returns an 'Err(...)' if something goes wrong
    let file = File::open(path)?;
    return Ok(io::BufReader::new(file).lines())
}

pub fn collect_commands() -> (Vec<Command>, usize)
{
    let mut inserts = 0;
    let mut commands: Vec<Command> = Vec::new();

    // Unwrap the BufferedReader from Ok(...)
    if let Ok(lines) = open_commands_file_with_buffered_reader()
    {
        for line in lines.map_while(Result::ok)
        {
            // Unwrap the Command from Some(...)
            if let Some(command) = compile_command(line)
            {
                if let Command::Insert { .. } = &command {
                    inserts += 1
                }
                commands.push(command);
            }
        }
    }
    commands.sort_unstable_by_key(|c| c.priority());
    return (commands, inserts);
}

fn compile_command(cmd: String) -> Option<Command>
{
    let mut components = cmd.split(',').map(|s| s.trim());
    let operation = components.next().expect("unknown split error");

    // Define the macro with a name
    macro_rules! expect_parameter {
        // Specify the parameters (...) of the macro
        ($parameter:literal) => { // Beginning of macro block
            // The macro expands into this block's contents
            components.next().unwrap_or_else(|| {
                panic!(
                    r#"
                    Missing parameter <{}> in command.
                    Expected format: {}
                    Given: {}
                    "#,
                    $parameter,
                    expected_format(operation),
                    cmd
                )
            })
        }; // End of macro block
    }

    // 'parse::<F>()' returns a 'Result<T, E>' enum, which is either 'Ok(T)' or 'Err(E)'
    // unwrap_or_else(<closure>) executes the given closure iff the enum is 'Err(E)'
    // Otherwise returning the 'T' value
    macro_rules! parse_u32 {
        ($value:expr, $parameter:literal) => {
            // "|| {}" defines a 'closure', there can be arguments within |...|
            // Underscore _ expresses to ignore the argument(s), e.g. |_|, |_, a, _|, etc.
            $value.parse::<u32>().unwrap_or_else(|_| {
                panic!(
                    r#"
                    Invalid parameter <{}>: '{}'.
                    Must be a valid unsigned 32-bit integer.
                    Expected format: {}
                    Given: {}
                    "#,
                    $parameter,
                    $value,
                    expected_format(operation),
                    cmd
                )
            })
        };
    }

    match operation
    {
        // Just consume "threads,<count>,<int>", since the way command are
        // parsed doesn't need it
        "threads" => {
            expect_parameter!("count");
            expect_parameter!("unknown");
            return None;
        }

        "insert" | "update" => {
            // Split and unwrap next returns &str, so convert into String
            let name = expect_parameter!("name").to_string();
            let salary = expect_parameter!("salary");
            let priority = expect_parameter!("priority");
            // Shadowing
            let salary = parse_u32!(salary, "salary");
            let priority = parse_u32!(priority, "priority");
            return match operation {
                "insert" => Some(Command::Insert { name, salary, priority }),
                "update" => Some(Command::Update { name, salary, priority }),
                _ => unreachable!()
            }
        }

        "delete" | "search" => {
            let name = expect_parameter!("name").to_string();
            let priority = expect_parameter!("priority");
            // Shadowing
            let priority = parse_u32!(priority, "priority");
            return match operation {
                "delete" => Some(Command::Delete { name, priority }),
                "search" => Some(Command::Search { name, priority }),
                _ => unreachable!()
            };
        }

        "print" => {
            let priority = expect_parameter!("priority");
            // Shadowing
            let priority = parse_u32!(priority, "priority");
            return Some(Command::Print { priority });
        }

        // "_" is what "default" would be in a C switch-case expression
        _ => {
            // Don't panic on empty lines
            if operation.is_empty() {
                return None;
            }
            panic!(
                r#"
                Unknown command: '{}'
                Valid commands:
                    insert,<name>,<salary>,<priority>
                    update,<name>,<salary>,<priority>
                    delete,<name>,<priority>
                    search,<name>,<priority>
                    print,<priority>

                Given: {}
                "#,
                operation,
                cmd
            )
        }
    }
}

fn expected_format(op: &str) -> &'static str {
    return match op {
        "insert" => "insert,<name>,<salary>,<priority>",
        "update" => "update,<name>,<salary>,<priority>",
        "delete" => "delete,<name>,<priority>",
        "search" => "search,<name>,<priority>",
        "print" => "print,<priority>",
        _ => "<not-applicable>",
    };
}
