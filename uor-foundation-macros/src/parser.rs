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
    /// String literal (host value).
    StringLiteral(String),
    /// Type declaration.
    TypeDecl {
        /// Type name.
        name: String,
        /// Type parameters (if any).
        type_params: Vec<String>,
        /// Constraint declarations: (kind, named_args).
        constraints: Vec<(String, Vec<(String, ParsedNode)>)>,
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
        /// Properties: (name, value).
        props: Vec<(String, ParsedNode)>,
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
    /// Try expression with recovery arms.
    TryExpr {
        /// Body expression.
        body: Box<ParsedNode>,
        /// Recovery arms: (failure_kind_name, recovery_expr).
        recover_arms: Vec<(String, ParsedNode)>,
    },
    /// Bounded recursion.
    Recurse {
        /// Function name.
        name: String,
        /// Parameter name.
        param: String,
        /// Measure variable name.
        measure: String,
        /// Base case arms: (predicate_name, result_expr).
        base_arms: Vec<(String, ParsedNode)>,
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
                "try" => self.parse_try(),
                "recurse" => self.parse_recurse(),
                "unfold" => self.parse_unfold(),
                _ => self.parse_expr(),
            },
            _ => self.parse_expr(),
        }
    }

    /// Parse a type declaration: `type Name [(params)] { Kind(arg: expr, ...); ... }`
    fn parse_type_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "type"
        let name = self.expect_ident()?;

        // Optional type params: (T, U, ...)
        let mut type_params = Vec::new();
        if self.peek() == &Token::LParen {
            self.advance(); // consume '('
            if self.peek() != &Token::RParen {
                type_params.push(self.expect_ident()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    type_params.push(self.expect_ident()?);
                }
            }
            self.expect(&Token::RParen)?;
        }

        self.expect(&Token::LBrace)?;
        let mut constraints = Vec::new();
        while self.peek() != &Token::RBrace {
            let kind = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let mut args = Vec::new();
            if self.peek() != &Token::RParen {
                let arg_name = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let arg_expr = self.parse_expr()?;
                args.push((arg_name, arg_expr));
                while self.peek() == &Token::Comma {
                    self.advance();
                    let arg_name = self.expect_ident()?;
                    self.expect(&Token::Colon)?;
                    let arg_expr = self.parse_expr()?;
                    args.push((arg_name, arg_expr));
                }
            }
            self.expect(&Token::RParen)?;
            self.expect(&Token::Semi)?;
            constraints.push((kind, args));
        }
        self.expect(&Token::RBrace)?;
        Ok(ParsedNode::TypeDecl {
            name,
            type_params,
            constraints,
        })
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

    /// Parse an effect declaration: `effect Name { prop: value; ... }`
    fn parse_effect_decl(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "effect"
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut props = Vec::new();
        while self.peek() != &Token::RBrace {
            let prop_name = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let value = self.parse_value()?;
            self.expect(&Token::Semi)?;
            props.push((prop_name, value));
        }
        self.expect(&Token::RBrace)?;

        Ok(ParsedNode::EffectDecl { name, props })
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

    /// Parse match expression: `match term { name => expr; ... }`
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
        // Default otherwise to a zero literal if not provided
        let otherwise = otherwise.unwrap_or_else(|| {
            Box::new(ParsedNode::Literal {
                value: 0,
                level: None,
            })
        });
        Ok(ParsedNode::Match {
            scrutinee,
            arms,
            otherwise,
        })
    }

    /// Parse try expression: `try term { recover Name => expr; ... }`
    fn parse_try(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "try"
        let body = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let mut recover_arms = Vec::new();
        while self.peek() != &Token::RBrace {
            let _recover_kw = self.expect_ident()?; // "recover"
            let kind_name = self.expect_ident()?;
            self.expect(&Token::FatArrow)?;
            let expr = self.parse_expr()?;
            self.expect(&Token::Semi)?;
            recover_arms.push((kind_name, expr));
        }
        self.expect(&Token::RBrace)?;
        Ok(ParsedNode::TryExpr {
            body: Box::new(body),
            recover_arms,
        })
    }

    /// Parse bounded recursion with multiple base arms.
    fn parse_recurse(&mut self) -> Result<ParsedNode, String> {
        self.advance(); // consume "recurse"
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let param = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        let _measure_kw = self.expect_ident()?; // "measure"
        let measure = self.expect_ident()?;

        // Parse one or more base arms
        let mut base_arms = Vec::new();
        while matches!(self.peek(), Token::Ident(s) if s == "base") {
            self.advance(); // consume "base"
            let pred = self.expect_ident()?;
            self.expect(&Token::FatArrow)?;
            let expr = self.parse_expr()?;
            base_arms.push((pred, expr));
        }
        if base_arms.is_empty() {
            return Err("recurse requires at least one base arm".to_string());
        }

        let _step_kw = self.expect_ident()?; // "step"
        self.expect(&Token::FatArrow)?;
        let step_expr = self.parse_expr()?;
        Ok(ParsedNode::Recurse {
            name,
            param: match param {
                ParsedNode::Variable(s) => s,
                _ => return Err("recurse parameter must be an identifier".to_string()),
            },
            measure,
            base_arms,
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

    /// Parse an expression (term).
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
            Token::StringLit(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(ParsedNode::StringLiteral(s))
            }
            Token::BoolLit(b) => {
                self.advance();
                Ok(ParsedNode::StringLiteral(b.to_string()))
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                // Lift and project are special forms
                if name == "lift" {
                    return self.parse_lift();
                }
                if name == "project" {
                    return self.parse_project();
                }
                // Any ident followed by '(' is an application
                if self.peek() == &Token::LParen {
                    self.advance(); // consume '('
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
                    Ok(ParsedNode::Variable(name))
                }
            }
            Token::LBrace => {
                // Set expression: { term, term, ... }
                self.advance(); // consume '{'
                let mut elems = Vec::new();
                if self.peek() != &Token::RBrace {
                    elems.push(self.parse_expr()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        elems.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBrace)?;
                Ok(ParsedNode::Application {
                    op: "__set".to_string(),
                    args: elems,
                })
            }
            other => Err(format!("Unexpected token in expression: {other:?}")),
        }
    }

    /// Parse a value (term or host literal).
    fn parse_value(&mut self) -> Result<ParsedNode, String> {
        self.parse_expr()
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
