mod codegen;
mod delta;
mod errors;
mod lexer;
mod line_editor;
mod parser;
mod typechecker;

use crate::{
    codegen::Translater,
    parser::Parser,
    typechecker::{FnRegister, TypeChecker},
};
use line_editor::{LineEditor, ReadLineOutput};

fn print_int(value: i64) {
    println!("value: {value}")
}

fn add_int(lhs: i64, rhs: i64) -> i64 {
    lhs + rhs
}

fn main() {
    let args = std::env::args();

    if args.len() > 1 {
        for arg in args.skip(1) {
            let contents = std::fs::read_to_string(arg).expect("couldn't find file");

            run_line(&contents, true);
        }
        return;
    }

    let mut line_editor = LineEditor::new();

    let mut debug_output = false;
    loop {
        match line_editor.read_line() {
            Ok(ReadLineOutput::Continue) => {
                continue;
            }
            Ok(ReadLineOutput::Break) => {
                break;
            }
            Ok(ReadLineOutput::Success(line)) => {
                if line == "exit" || line == "quit" {
                    break;
                } else if line == "debug" {
                    debug_output = !debug_output;
                } else {
                    run_line(&line, debug_output);
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

fn run_line(line: &str, debug_output: bool) {
    if debug_output {
        println!("line: {line}");
    }

    let mut parser = Parser::new(line.as_bytes(), 0, 0);

    parser.parse();

    for error in &parser.errors {
        println!("error: {:?}", error);
    }

    let mut typechecker = TypeChecker::new(parser.delta.ast_nodes.len());
    typechecker.register_fn("print_int", print_int);
    typechecker.register_fn("add_int", add_int);
    typechecker.typecheck(&parser.delta);

    for error in &typechecker.errors {
        println!("error: {:?}", error);
    }

    let result = &parser.delta;

    if debug_output {
        println!();
        println!("parse result:");
        result.print();
    }
    let mut idx = 0;

    if debug_output {
        println!();
        println!("typed nodes:");
        while idx < result.ast_nodes.len() {
            println!(
                "  {}: {:?} ({})",
                idx,
                result.ast_nodes[idx],
                typechecker.stringify_type(typechecker.node_types[idx])
            );
            idx += 1;
        }
    }

    let mut translater = Translater::new();

    let mut output = translater.translate(&parser.delta, &typechecker);

    if debug_output {
        println!();
        println!("===stdout===");
    }
    let result = output.eval(&typechecker.functions);
    if debug_output {
        println!("============");
        println!();
        output.debug_print(&typechecker);
        println!();
    }
    println!(
        "result -> {} ({})",
        result.0,
        typechecker.stringify_type(result.1)
    );
}
