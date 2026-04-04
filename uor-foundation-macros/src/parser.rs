//! Recursive-descent parser for the UOR EBNF term grammar.

use crate::lexer::{self, Token};

/// Parsed AST produced by the parser. This is the proc-macro-internal
/// representation; codegen converts it to `uor_foundation::enforcement` types.
#[derive(Debug, Clone)]
pub enum ParsedNode {
    /// Integer literal with optional quantum level annotation.
    Literal {
        /// The integer value.
        value: u64,
        /// Quantum level name (e.g., "Q0", "Q7"), or None for default.
        level: Option<String>,
    },
    /// Variable reference.
    Variable(String),
    /// Operation application.
    Application {
        /// Operation name (e.g., "add", "neg").
        op: String,
        /// Arguments.
        args: Vec<ParsedNode>,
    },
    /// Lift expression.
    Lift {
        /// The operand.
        operand: Box<ParsedNode>,
        /// Target level name.
        target: String,
    },
    /// Project expression.
    Project {
        /// The operand.
        operand: Box<ParsedNode>,
        /// Target level name.
        target: String,
    },
    /// Type declaration.
    TypeDecl {
        /// Type name.
        name: String,
        /// Constraint declarations (kind, expression).
        constraints: Vec<(String, ParsedNode)>,
    },
    /// Named binding.
    Binding {
        /// Binding name.
        name: String,
        /// Type annotation.
        type_name: String,
        /// Value expression.
        value: Box<ParsedNode>,
    },
    /// Ground assertion.
    Assertion {
        /// Left-hand side.
        lhs: Box<ParsedNode>,
        /// Right-hand side.
        rhs: Box<ParsedNode>,
    },
    /// Effect declaration.
    EffectDecl {
        /// Effect name.
        name: String,
        /// Target fiber set.
        target: Vec<u64>,
        /// Budget delta.
        delta: i64,
        /// Commutation flag.
        commutes: bool,
    },
    /// Source declaration.
    SourceDecl {
        /// Source name.
        name: String,
        /// Type name.
        type_name: String,
        /// Grounding map name.
        via: String,
    },
    /// Sink declaration.
    SinkDecl {
        /// Sink name.
        name: String,
        /// Type name.
        type_name: String,
        /// Projection map name.
        via: String,
    },
    /// Match expression.
    Match {
        /// Scrutinee variable.
        scrutinee: String,
        /// Arms: (pattern_name, result_expr).
        arms: Vec<(String, ParsedNode)>,
        /// Otherwise arm.
        otherwise: Box<ParsedNode>,
    },
    /// Bounded recursion.
    Recurse {
        /// Function name.
        name: String,
        /// Parameter name.
        param: String,
        /// Measure variable name.
        measure: String,
        /// Base case predicate name.
        base_pred: String,
        /// Base case result.
        base_expr: Box<ParsedNode>,
        /// Recursive step expression.
        step_expr: Box<ParsedNode>,
    },
    /// Stream unfold.
    Unfold {
        /// Stream name.
        name: String,
        /// Type name.
        type_name: String,
        /// Seed expression.
        seed: Box<ParsedNode>,
    },
}

/// Parser state.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            other => Err(format!("Expected identifier, got {other:?}")),
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let got = self.advance();
        if &got == expected {
            Ok(())
        } else {
            Err(format!("Expected {expected:?}, got {got:?}"))
        }
    }

    /// Parse a statement (the top-level entry point).
    fn parse_statement(&mut self) -> Result<ParsedNode, String> {
        match self.peek().clone() {
            Token::Ident(ref kw) => match kw.as_str() {
                "type" => self.parse_type_decl(),
                "let" => self.parse_binding(),
                "assert" => self.parse_assertion(),
                "effect" => self.parse_effect_decl(),
                "source" => self.parse_source_decl(),
                "sink" => self.parse_sink_decl(),
                "match" => self.parse_match(),
                "recurse" => self.parse_recurse(),
                "unfold" => self.parse_unfold(),
                _ => self.parse_expr(),
            },
            _ => self.parse_expr(),
        }
    }

    /// Parse a type declaration: `type Name { kind: expr; ... }`
    fn parse_type_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "type"
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut constraints = Vec::new();
        while self.peek() != &Token::RBrace {
            let kind = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let expr = self.parse_expr()?;
            self.expect(&Token::Semi)?;
            constraints.push((kind, expr));
        }
        self.expect(&Token::RBrace)?;
        Ok(ParsedNode::TypeDecl { name, constraints })
    }

    /// Parse a binding: `let name : Type = expr ;`
    fn parse_binding(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "let"
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let type_name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(ParsedNode::Binding {
            name,
            type_name,
            value: Box::new(value),
        })
    }

    /// Parse an assertion: `assert lhs = rhs ;`
    fn parse_assertion(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "assert"
        let lhs = self.parse_expr()?;
        self.expect(&Token::Eq)?;
        let rhs = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(ParsedNode::Assertion {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    /// Parse an effect declaration.
    fn parse_effect_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "effect"
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        // target: { int, int, ... };
        let _target_kw = self.expect_ident()?; // "target"
        self.expect(&Token::Colon)?;
        self.expect(&Token::LBrace)?;
        let mut target = Vec::new();
        while let Token::IntLit(v) = self.peek().clone() {
            target.push(v);
            self.advance();
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        self.expect(&Token::Semi)?;

        // delta: INT;
        let _delta_kw = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let delta_val = match self.advance() {
            Token::IntLit(v) => v as i64,
            other => return Err(format!("Expected integer for delta, got {other:?}")),
        };
        self.expect(&Token::Semi)?;

        // commutes: BOOL;
        let _commutes_kw = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let commutes = match self.advance() {
            Token::BoolLit(b) => b,
            other => return Err(format!("Expected bool for commutes, got {other:?}")),
        };
        self.expect(&Token::Semi)?;
        self.expect(&Token::RBrace)?;

        Ok(ParsedNode::EffectDecl {
            name,
            target,
            delta: delta_val,
            commutes,
        })
    }

    /// Parse source declaration: `source name : Type via grounding ;`
    fn parse_source_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "source"
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let type_name = self.expect_ident()?;
        let _via = self.expect_ident()?; // "via"
        let via = self.expect_ident()?;
        self.expect(&Token::Semi)?;
        Ok(ParsedNode::SourceDecl {
            name,
            type_name,
            via,
        })
    }

    /// Parse sink declaration: `sink name : Type via projection ;`
    fn parse_sink_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "sink"
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let type_name = self.expect_ident()?;
        let _via = self.expect_ident()?;
        let via = self.expect_ident()?;
        self.expect(&Token::Semi)?;
        Ok(ParsedNode::SinkDecl {
            name,
            type_name,
            via,
        })
    }

    /// Parse match expression.
    fn parse_match(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "match"
        let scrutinee = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        let mut otherwise = None;
        while self.peek() != &Token::RBrace {
            let arm_name = self.expect_ident()?;
            if arm_name == "otherwise" {
                self.expect(&Token::FatArrow)?;
                let expr = self.parse_expr()?;
                self.expect(&Token::Semi)?;
                otherwise = Some(Box::new(expr));
            } else {
                self.expect(&Token::FatArrow)?;
                let expr = self.parse_expr()?;
                self.expect(&Token::Semi)?;
                arms.push((arm_name, expr));
            }
        }
        self.expect(&Token::RBrace)?;
        let otherwise =
            otherwise.ok_or_else(|| "Match expression missing 'otherwise' arm".to_string())?;
        Ok(ParsedNode::Match {
            scrutinee,
            arms,
            otherwise,
        })
    }

    /// Parse bounded recursion.
    fn parse_recurse(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "recurse"
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let param = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        let _measure_kw = self.expect_ident()?; // "measure"
        let measure = self.expect_ident()?;
        let _base_kw = self.expect_ident()?; // "base"
        let base_pred = self.expect_ident()?;
        self.expect(&Token::FatArrow)?;
        let base_expr = self.parse_expr()?;
        let _step_kw = self.expect_ident()?; // "step"
        self.expect(&Token::FatArrow)?;
        let step_expr = self.parse_expr()?;
        Ok(ParsedNode::Recurse {
            name,
            param,
            measure,
            base_pred,
            base_expr: Box::new(base_expr),
            step_expr: Box::new(step_expr),
        })
    }

    /// Parse stream unfold.
    fn parse_unfold(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "unfold"
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let type_name = self.expect_ident()?;
        let _from = self.expect_ident()?; // "from"
        let seed = self.parse_expr()?;
        Ok(ParsedNode::Unfold {
            name,
            type_name,
            seed: Box::new(seed),
        })
    }

    /// Parse an expression.
    fn parse_expr(&mut self) -> Result<ParsedNode, String> {
        match self.peek().clone() {
            Token::IntLit(v) => {
                self.advance();
                // Check for @Level annotation
                if self.peek() == &Token::At {
                    self.advance();
                    let level = self.expect_ident()?;
                    Ok(ParsedNode::Literal {
                        value: v,
                        level: Some(level),
                    })
                } else {
                    Ok(ParsedNode::Literal {
                        value: v,
                        level: None,
                    })
                }
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                // Check if this is an operation (followed by '(')
                if is_operation(&name) || name == "lift" || name == "project" {
                    self.advance();
                    if name == "lift" {
                        return self.parse_lift();
                    }
                    if name == "project" {
                        return self.parse_project();
                    }
                    self.expect(&Token::LParen)?;
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.parse_expr()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(ParsedNode::Application { op: name, args })
                } else {
                    self.advance();
                    Ok(ParsedNode::Variable(name))
                }
            }
            other => Err(format!("Unexpected token in expression: {other:?}")),
        }
    }

    /// Parse lift(operand, Level).
    fn parse_lift(&mut self) -> Result<ParsedNode, String> {
        self.expect(&Token::LParen)?;
        let operand = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let target = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        Ok(ParsedNode::Lift {
            operand: Box::new(operand),
            target,
        })
    }

    /// Parse project(operand, Level).
    fn parse_project(&mut self) -> Result<ParsedNode, String> {
        self.expect(&Token::LParen)?;
        let operand = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let target = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        Ok(ParsedNode::Project {
            operand: Box::new(operand),
            target,
        })
    }
}

/// Check if an identifier is a known operation name.
fn is_operation(name: &str) -> bool {
    matches!(
        name,
        "neg" | "bnot" | "succ" | "pred" | "add" | "sub" | "mul" | "xor" | "and" | "or"
    )
}

/// Parse the input string into a `ParsedNode`.
///
/// # Errors
///
/// Returns a string error if tokenization or parsing fails.
pub fn parse(input: &str) -> Result<ParsedNode, String> {
    let tokens = lexer::tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}
