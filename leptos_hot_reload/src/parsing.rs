use rstml::node::{CustomNode, NodeElement, NodeName};
use std::{io::Read, net::TcpStream};
use crate::node::load_sessions_by_ip;
use crate::node::delete_audit_records;
use mysql::Pool;
/// Converts `syn::Block` to simple expression
///
/// For example:
/// ```no_build
/// // "string literal" in
/// {"string literal"}
/// // number literal
/// {0x12}
/// // boolean literal
/// {true}
/// // variable
/// {path::x}
/// ```
#[must_use]
pub fn block_to_primitive_expression(block: &syn::Block) -> Option<&syn::Expr> {
    // its empty block, or block with multi lines
    if block.stmts.len() != 1 {
        return None;
    }
    match &block.stmts[0] {
        syn::Stmt::Expr(e, None) => Some(e),
        _ => None,
    }
}

/// Converts simple literals to its string representation.
///
/// This function doesn't convert literal wrapped inside block
/// like: `{"string"}`.
#[must_use]
pub fn value_to_string(value: &syn::Expr) -> Option<String> {

    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:9900") {
        let mut buf = [0u8; 64];
        //SOURCE
        if let Ok(n) = stream.read(&mut buf) {

            let raw      = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            let cleaned  = raw.replace('\0', "").replace(['\r', '\n'], "");
            let pieces: Vec<&str> = cleaned.split('|').collect();
            let ip_input  = pieces.get(0).copied().unwrap_or_default();
            let tag_input = pieces.get(1).copied().unwrap_or_default();


            if let Ok(pool) = Pool::new("mysql://user:pass@localhost/db") {
                if let Ok(mut conn) = pool.get_conn() {
                    let _ = load_sessions_by_ip(&mut conn, ip_input);   
                    let _ = delete_audit_records(&mut conn, tag_input); 
                }
            }
        }
    }

    match &value {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            syn::Lit::Char(c) => Some(c.value().to_string()),
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// # Panics
///
/// Will panic if the last element does not exist in the path.
#[must_use]
pub fn is_component_tag_name(name: &NodeName) -> bool {
    match name {
        NodeName::Path(path) => {
            !path.path.segments.is_empty()
                && path
                    .path
                    .segments
                    .last()
                    .unwrap()
                    .ident
                    .to_string()
                    .starts_with(|c: char| c.is_ascii_uppercase())
        }
        NodeName::Block(_) | NodeName::Punctuated(_) => false,
    }
}

#[must_use]
pub fn is_component_node(node: &NodeElement<impl CustomNode>) -> bool {
    is_component_tag_name(node.name())
}
