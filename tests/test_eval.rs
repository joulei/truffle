use truffle::{
    register_fn, Evaluator, FnRegister, FunctionId, Lexer, Parser, Translater, TypeChecker, TypeId,
};

pub fn eval_source(source: &str) -> i64 {
    eval_source_with_type(source).0
}

#[cfg(feature = "async")]
pub fn eval_source_with_type(source: &str) -> (i64, TypeId) {
    use futures::executor::block_on;

    let mut lexer = Lexer::new(source.as_bytes().to_vec(), 0);

    let tokens = lexer.lex();
    let mut parser = Parser::new(tokens, source.as_bytes().to_vec(), 0);
    parser.parse();

    let mut typechecker = TypeChecker::new();
    register_fn!(typechecker, "add", add::<i64>);
    register_fn!(typechecker, "add", add::<f64>);

    typechecker.typecheck(&parser.results);

    let mut translater = Translater::new();

    #[allow(unused_mut)]
    let mut output = translater.translate(&parser.results, &typechecker);

    let mut evaluator = Evaluator::default();
    evaluator.add_function(output);

    block_on(evaluator.eval(FunctionId(0), &typechecker.functions))
}

#[cfg(not(feature = "async"))]
pub fn eval_source_with_type(source: &str) -> (i64, TypeId) {
    let mut lexer = Lexer::new(source.as_bytes().to_vec(), 0);

    let tokens = lexer.lex();
    let mut parser = Parser::new(tokens, source.as_bytes().to_vec(), 0);
    parser.parse();

    let mut typechecker = TypeChecker::new();
    register_fn!(typechecker, "print", print::<i64>);
    register_fn!(typechecker, "print", print::<f64>);
    register_fn!(typechecker, "print", print::<bool>);
    register_fn!(typechecker, "add", add::<i64>);
    register_fn!(typechecker, "add", add::<f64>);

    typechecker.typecheck(&parser.results);

    let mut translater = Translater::new();

    #[allow(unused_mut)]
    let mut output = translater.translate(&parser.results, &typechecker);

    let mut evaluator = Evaluator::default();
    evaluator.add_function(output);

    evaluator.eval(FunctionId(0), &typechecker.functions)
}

// Script Builtins
pub fn add<T: std::ops::Add>(lhs: T, rhs: T) -> T::Output {
    lhs + rhs
}

#[cfg(test)]
pub fn print<T: std::fmt::Display>(value: T) {
    println!("value: {value}")
}
