//! Surface syntax serializer for round-trip property.
//!
//! Converts a `ParsedNode` back to canonical EBNF surface syntax.

use crate::parser::ParsedNode;

/// Serialize a parsed node to its canonical EBNF surface syntax string.
pub fn serialize(node: &ParsedNode) -> String {
    match node {
        ParsedNode::Literal { value, level } => match level {
            Some(l) => format!("{value}@{l}"),
            None => format!("{value}"),
        },
        ParsedNode::StringLiteral(s) => format!("\"{s}\""),
        ParsedNode::Variable(name) => name.clone(),
        ParsedNode::Application { op, args } => {
            let arg_strs: Vec<String> = args.iter().map(serialize).collect();
            format!("{}({})", op, arg_strs.join(", "))
        }
        ParsedNode::Lift { operand, target } => {
            format!("lift({}, {})", serialize(operand), target)
        }
        ParsedNode::Project { operand, target } => {
            format!("project({}, {})", serialize(operand), target)
        }
        ParsedNode::TypeDecl {
            name,
            type_params,
            constraints,
        } => {
            let params = if type_params.is_empty() {
                String::new()
            } else {
                format!("({})", type_params.join(", "))
            };
            let mut s = format!("type {name}{params} {{");
            for (kind, args) in constraints {
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|(n, e)| format!("{n}: {}", serialize(e)))
                    .collect();
                s.push_str(&format!(" {kind}({});", arg_strs.join(", ")));
            }
            s.push_str(" }");
            s
        }
        ParsedNode::Binding {
            name,
            type_name,
            value,
        } => {
            format!("let {name} : {type_name} = {};", serialize(value))
        }
        ParsedNode::Assertion { lhs, rhs } => {
            format!("assert {} = {};", serialize(lhs), serialize(rhs))
        }
        ParsedNode::EffectDecl { name, props } => {
            let mut s = format!("effect {name} {{");
            for (prop_name, value) in props {
                s.push_str(&format!(" {prop_name}: {};", serialize(value)));
            }
            s.push_str(" }");
            s
        }
        ParsedNode::SourceDecl {
            name,
            type_name,
            via,
        } => {
            format!("source {name} : {type_name} via {via};")
        }
        ParsedNode::SinkDecl {
            name,
            type_name,
            via,
        } => {
            format!("sink {name} : {type_name} via {via};")
        }
        ParsedNode::Match {
            scrutinee,
            arms,
            otherwise,
        } => {
            let mut s = format!("match {scrutinee} {{");
            for (pat, expr) in arms {
                s.push_str(&format!(" {pat} => {};", serialize(expr)));
            }
            s.push_str(&format!(" otherwise => {};", serialize(otherwise)));
            s.push_str(" }");
            s
        }
        ParsedNode::TryExpr { body, recover_arms } => {
            let mut s = format!("try {} {{", serialize(body));
            for (kind, expr) in recover_arms {
                s.push_str(&format!(" recover {kind} => {};", serialize(expr)));
            }
            s.push_str(" }");
            s
        }
        ParsedNode::Recurse {
            name,
            param,
            measure,
            base_arms,
            step_expr,
        } => {
            let mut s = format!("recurse {name}({param}) measure {measure}");
            for (pred, expr) in base_arms {
                s.push_str(&format!(" base {pred} => {}", serialize(expr)));
            }
            s.push_str(&format!(" step => {}", serialize(step_expr)));
            s
        }
        ParsedNode::Unfold {
            name,
            type_name,
            seed,
        } => {
            format!("unfold {name} : {type_name} from {}", serialize(seed))
        }
    }
}
