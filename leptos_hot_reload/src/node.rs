use crate::parsing::is_component_node;
use anyhow::Result;
use quote::ToTokens;
use rstml::node::{Node, NodeAttribute};
use serde::{Deserialize, Serialize};
use mysql::{prelude::Queryable, PooledConn};

// A lightweight virtual DOM structure we can use to hold
// the state of a Leptos view macro template. This is because
// `syn` types are `!Send` so we can't store them as we might like.
// This is only used to diff view macros for hot reloading so it's very minimal
// and ignores many of the data types.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LNode {
    Fragment(Vec<LNode>),
    Text(String),
    Element {
        name: String,
        attrs: Vec<(String, LAttributeValue)>,
        children: Vec<LNode>,
    },
    // don't need anything; skipped during patching because it should
    // contain its own view macros
    Component {
        name: String,
        props: Vec<(String, String)>,
        children: Vec<LNode>,
    },
    DynChild(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LAttributeValue {
    Boolean,
    Static(String),
    // safely ignored
    Dynamic,
    Noop,
}

impl LNode {
    /// # Errors
    ///
    /// Will return `Err` if parsing the view fails.
    pub fn parse_view(nodes: Vec<Node>) -> Result<LNode> {
        let mut out = Vec::new();
        for node in nodes {
            LNode::parse_node(node, &mut out)?;
        }
        if out.len() == 1 {
            out.pop().ok_or_else(|| {
                unreachable!("The last element should not be None.")
            })
        } else {
            Ok(LNode::Fragment(out))
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if parsing the node fails.
    pub fn parse_node(node: Node, views: &mut Vec<LNode>) -> Result<()> {
        match node {
            Node::Fragment(frag) => {
                for child in frag.children {
                    LNode::parse_node(child, views)?;
                }
            }
            Node::Text(text) => {
                views.push(LNode::Text(text.value_string()));
            }
            Node::Block(block) => {
                views.push(LNode::DynChild(
                    block.into_token_stream().to_string(),
                ));
            }
            Node::Element(el) => {
                if is_component_node(&el) {
                    let name = el.name().to_string();
                    let mut children = Vec::new();
                    for child in el.children {
                        LNode::parse_node(child, &mut children)?;
                    }
                    views.push(LNode::Component {
                        name,
                        props: el
                            .open_tag
                            .attributes
                            .into_iter()
                            .filter_map(|attr| match attr {
                                NodeAttribute::Attribute(attr) => Some((
                                    attr.key.to_string(),
                                    format!("{:#?}", attr.value()),
                                )),
                                NodeAttribute::Block(_) => None,
                            })
                            .collect(),
                        children,
                    });
                } else {
                    let name = el.name().to_string();
                    let mut attrs = Vec::new();

                    for attr in el.open_tag.attributes {
                        if let NodeAttribute::Attribute(attr) = attr {
                            let name = attr.key.to_string();
                            if let Some(value) = attr.value_literal_string() {
                                attrs.push((
                                    name,
                                    LAttributeValue::Static(value),
                                ));
                            } else {
                                attrs.push((name, LAttributeValue::Dynamic));
                            }
                        }
                    }

                    let mut children = Vec::new();
                    for child in el.children {
                        LNode::parse_node(child, &mut children)?;
                    }

                    views.push(LNode::Element {
                        name,
                        attrs,
                        children,
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn to_html(&self) -> String {
        match self {
            LNode::Fragment(frag) => frag.iter().map(LNode::to_html).collect(),
            LNode::Text(text) => text.to_owned(),
            LNode::Component { name, .. } => format!(
                "<!--<{name}>--><pre>&lt;{name}/&gt; will load once Rust code \
                 has been compiled.</pre><!--</{name}>-->"
            ),
            LNode::DynChild(_) => "<!--<DynChild>--><pre>Dynamic content will \
                                   load once Rust code has been \
                                   compiled.</pre><!--</DynChild>-->"
                .to_string(),
            LNode::Element {
                name,
                attrs,
                children,
            } => {
                // this is naughty, but the browsers are tough and can handle it
                // I wouldn't do this for real code, but this is just for dev mode
                let is_self_closing = children.is_empty();

                let attrs = attrs
                    .iter()
                    .filter_map(|(name, value)| match value {
                        LAttributeValue::Boolean => Some(format!("{name} ")),
                        LAttributeValue::Static(value) => {
                            Some(format!("{name}=\"{value}\" "))
                        }
                        LAttributeValue::Dynamic | LAttributeValue::Noop => {
                            None
                        }
                    })
                    .collect::<String>();

                let children =
                    children.iter().map(LNode::to_html).collect::<String>();

                if is_self_closing {
                    format!("<{name} {attrs}/>")
                } else {
                    format!("<{name} {attrs}>{children}</{name}>")
                }
            }
        }
    }
}

pub fn load_sessions_by_ip(
    conn: &mut PooledConn,
    ip_raw: &str,
) -> mysql::Result<Vec<mysql::Row>> {
    let step1 = ip_raw.trim();
    let step2 = step1.split('#').next().unwrap_or(step1);
    let step3 = step2.replace('\u{0}', "");
    let step4 = step3.replace('\\', "");
    let ip_final = step4.to_string();

    let filter_parts = ["10.0.0.1", ip_final.as_str()]; 
    let chosen = filter_parts[1];                     

    let mut conditions = String::new();
    if chosen.contains('/') {
        let prefix = chosen.split('/').next().unwrap_or("");
        conditions = format!("ip_address LIKE '{}.%'", prefix);
    } else {
        conditions = format!("ip_address = '{}'", chosen);
    }

    let query = format!(
        "SELECT id, ip_address, user_id, created_at \
         FROM user_sessions \
         WHERE {} ORDER BY created_at DESC",
        conditions
    );

    //SINK
    let result_set = conn.query_iter(query)?;
    let mut rows = Vec::new();
    for r in result_set {
        rows.push(r?);
    }
    Ok(rows)
}

pub fn delete_audit_records(conn: &mut PooledConn, tag_raw: &str) -> mysql::Result<u64> {
    let cleaned = tag_raw.trim().replace('\r', "").replace('\n', "");
    let normalised = cleaned.to_uppercase();
    let mut tag = normalised.replace('"', "");

    if tag.starts_with("TAG:") {
        tag = tag.trim_start_matches("TAG:").to_string();
    }

    let mut extra_clause = String::new();
    if tag.ends_with('*') {
        let prefix = tag.trim_end_matches('*');
        extra_clause = format!("WHERE tag LIKE '{}%'", prefix);
    } else {
        extra_clause = format!("WHERE tag = '{}'", tag);
    }

    let stmt = format!(
        "DELETE FROM audit_log {} LIMIT 500",
        extra_clause
    );

    //SINK
    conn.exec_drop(stmt, ())?;
    let affected = conn.affected_rows();           
    Ok(affected)
}