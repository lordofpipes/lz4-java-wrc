mod arguments;
mod command;
mod read_counter;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum ReturnCode {
    Ok = 0,
    ErrorCommand = 1,
    ErrorParsing = 2,
    ErrorUnknown = 255,
}

fn main() {
    let return_code = match arguments::parse_cli() {
        Err(s) => {
            eprintln!("ERROR: while parsing arguments: {}", s);
            ReturnCode::ErrorParsing
        }
        Ok(args) => {
            let command =
                command::Command::new(args.lz4jb_context, args.mode, args.keep_input, args.force);
            args.files
                .iter()
                .map(|f| (f, command.run(f)))
                .map(|(f, res)| match res {
                    Err(err) => {
                        eprintln!("ERROR: could not {} from {}: {}", args.mode, f.file_in, err);
                        ReturnCode::ErrorCommand
                    }
                    _ => ReturnCode::Ok,
                })
                .max()
                .unwrap_or(ReturnCode::ErrorUnknown)
        }
    };
    std::process::exit(return_code as i32);
}
