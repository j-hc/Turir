use std::{
    io::{stdout, BufWriter, Read, Write},
    process::ExitCode,
};
use turir::{
    compiler::Compiler,
    parser::{self, Dir, ParseErr, Program},
};

fn read_source(s: &str) -> std::io::Result<Vec<u8>> {
    let mut f = std::fs::File::open(s)?;
    let sz = f.metadata()?.len() as usize;

    // hackish way to put a new line at the end of source code
    let mut buf = Vec::with_capacity(sz + 1);
    f.read_to_end(&mut buf)?;
    buf.push(b'\n');
    Ok(buf)
}

#[allow(unused_must_use)]
fn tape_print(tape: &[&str], head: usize, sink: &mut impl Write) {
    write!(sink, "[ ");
    let (last, t) = tape.split_last().expect("tape cannot be empty");
    for t in t {
        write!(sink, "{t} ");
    }
    write!(sink, "{last} ]");
    write!(sink, "\n  ");
    for _ in 0..head {
        write!(sink, "  ");
    }
    writeln!(sink, "^");
}

#[allow(unused_must_use)]
fn execute_program<'c, 'k>(program: Program<'_>) -> Result<(), ParseErr<'c, 'k>> {
    let mut sink = BufWriter::new(stdout().lock());

    for run in program.runs {
        writeln!(sink, "{run}");

        let mut tape = run.tape;
        let mut state = run.state;
        let mut head = 0;

        let jmp_instr = |state, read| match program
            .program
            .iter()
            .find(|instr| instr.state == state && instr.read == read)
        {
            Some(s) => s,
            None => {
                eprintln!("State '{state}' and read '{read}' combination is not defined");
                std::process::exit(0);
            }
        };

        // tape has got at least one element
        let mut instr = jmp_instr(state, tape[0]);
        let default_sym = tape.last().unwrap().to_string();
        loop {
            writeln!(sink, "{}", instr);

            tape_print(&tape, head, &mut sink);

            tape[head] = instr.write;
            state = instr.next_state;

            match instr.dir {
                Dir::Left => head -= 1,
                Dir::Right => head += 1,
            }
            if program.halt_syms.iter().any(|&c| instr.next_state == c) {
                break;
            }
            if tape.len() <= head {
                tape.push(&default_sym)
            }
            sink.flush();

            instr = jmp_instr(state, tape[head])
        }
        tape_print(&tape, head, &mut sink);
        writeln!(sink, " -- HALT -- with {}", state);
        writeln!(sink);
    }

    sink.flush();

    Ok(())
}

enum CmdArg {
    Run,
    Compile,
}

const USAGE: &str = "\trun <source code>.tur\n\tcompile <source code>.tur";

fn parse_args() -> Option<(CmdArg, String)> {
    let mut args = std::env::args();
    let r = args.next()?;

    let cmd = match args.next().as_deref() {
        Some("compile") => CmdArg::Compile,
        Some("run") => CmdArg::Run,
        Some(c) => {
            eprintln!("{c} is not a valid command\nUsage: {r}\n{USAGE}");
            return None;
        }
        None => {
            eprintln!("{USAGE}");
            return None;
        }
    };
    let f = match args.next() {
        Some(f) => f,
        None => {
            eprintln!("No source file is provided\nUsage: {r}\n{USAGE}");
            return None;
        }
    };

    Some((cmd, f))
}

fn main() -> ExitCode {
    let Some((cmd, file)) = parse_args() else {
        return ExitCode::FAILURE;
    };
    let file: &'static str = Box::leak(file.into_boxed_str());
    let content = read_source(file).unwrap();

    let program = match parser::parse_source(&content, file) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    match cmd {
        CmdArg::Run => {
            if let Err(err) = execute_program(program) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        }
        CmdArg::Compile => {
            let mut compiler = Compiler::default();
            compiler.compile_program(program);
        }
    }

    ExitCode::SUCCESS
}
