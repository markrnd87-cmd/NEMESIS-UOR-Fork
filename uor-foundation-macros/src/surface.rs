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
        ParsedNode::TypeDecl { name, constraints } => {
            let mut s = format!("type {name} {{");
            for (kind, expr) in constraints {
                s.push_str(&format!(" {kind}: {};", serialize(expr)));
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
        ParsedNode::EffectDecl {
            name,
            target,
            delta,
            commutes,
        } => {
            let targets: Vec<String> = target.iter().map(|t| format!("{t}")).collect();
            format!(
                "effect {name} {{ target: {{{}}}; delta: {delta}; commutes: {commutes}; }}",
                targets.join(", ")
            )
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
        ParsedNode::Recurse {
            name,
            param,
            measure,
            base_pred,
            base_expr,
            step_expr,
        } => {
            format!(
                "recurse {name}({param}) measure {measure} base {base_pred} => {} step => {}",
                serialize(base_expr),
                serialize(step_expr)
            )
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
