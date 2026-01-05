use rhai::{Engine, Scope, AST};

pub fn process_runtime_input(script: String) -> String {
    let engine = Engine::new();
    let mut scope = Scope::new();
    scope.push("base", 40_i64);

    let ast: AST = match engine.compile_expression(&script) {
        Ok(a) => a,
        Err(e) => return e.to_string(),
    };

    //SINK
    match engine.run_ast_with_scope(&mut scope, &ast) {
        Ok(()) => "script executed without error".to_string(),
        Err(e) => e.to_string(),
    }
}
