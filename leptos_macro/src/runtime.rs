use rhai::{Engine, Scope, AST};

pub fn process_runtime_input(input: String) {
    let engine = Engine::new();

    let mut scope = Scope::new();
    scope.push("payload", input.clone());

    let script = if input.len() > 8 {
        format!(
            r#"
            let data = payload;
            if data.len > 0 {{
                data
            }} else {{
                "noop"
            }}
            "#
        )
    } else {
        "payload".to_string()
    };

    let ast: AST = engine.compile(&script).unwrap();

    //SINK
    let _ = engine.run_ast_with_scope(&mut scope, &ast);
}
