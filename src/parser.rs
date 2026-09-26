use crate::{
    expr::Expr,
    lox_error::LoxError,
    stmt::Stmt,
    token::{Token, TokenType, TokenValue},
};

const FUNCTION: &str = "function";
const METHOD: &str = "method";

static EOF_TOKEN: Token = Token {
    token_type: TokenType::Eof,
    lexeme: String::new(),
    literal: None,
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        // Make sure the tokens Vec ends with an Eof
        assert!(
            tokens.last() == Some(&EOF_TOKEN),
            "The tokens vector must end with an Eof token"
        );

        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxError> {
        parse_program(self)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, LoxError> {
        parse_expression(self)
    }

    fn advance(&mut self) -> &Token {
        if !(self.is_at_end()) {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current == self.tokens.len() - 1
    }

    fn peek(&self) -> &Token {
        // SAFETY: self.current is only ever incremented by self.advance,
        // and only if it is not already pointing to the last token
        // Also, the Parser panic()s if the tokens Vec does not contain one
        // (EOF) token
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        match self.current {
            0 => &EOF_TOKEN,
            // SAFETY: self.current is > 0 here and only advanced through
            // advance, which ensures it is does not go out of bounds
            _ => self.tokens.get(self.current - 1).unwrap(),
        }
    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        // SAFETY: self.current can only be 0 to tokens.len() and
        // tokens has at least one element via new()
        if token_types.contains(&self.tokens.get(self.current).unwrap().token_type) {
            self.advance();
            return true;
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        self.peek().token_type == *token_type
    }

    fn consume(&mut self, token_type: &TokenType) -> Result<&Token, LoxError> {
        if self.peek().token_type == *token_type {
            self.advance();
            Ok(self.previous())
        } else {
            Err(LoxError::SyntaxError(format!("{token_type:?} expected")))
        }
    }
}

// Implementation of the different steps of the grammar

// program        → declaration* EOF ;
fn parse_program(parser: &mut Parser) -> Result<Vec<Stmt>, LoxError> {
    let mut program = vec![];
    while !parser.is_at_end() {
        program.push(parse_declaration(parser)?);
    }

    Ok(program)
}

// declaration    → funDecl
//                | varDecl
//                | statement ;
fn parse_declaration(parser: &mut Parser) -> Result<Stmt, LoxError> {
    if parser.matches(&[TokenType::Fun]) {
        return parse_function(parser, FUNCTION);
    }
    if parser.matches(&[TokenType::Var]) {
        return parse_var_declaration(parser);
    }
    parse_statement(parser)
}

// funDecl        → "fun" function ;
// function       → IDENTIFIER "(" parameters? ")" block ;
// parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
fn parse_function(parser: &mut Parser, kind: &str) -> Result<Stmt, LoxError> {
    let Ok(name) = parser.consume(&TokenType::Identifier).cloned() else {
        return Err(LoxError::SyntaxError(format!("Expect {kind} name")));
    };
    let Ok(_) = parser.consume(&TokenType::LeftParen) else {
        return Err(LoxError::SyntaxError(format!(
            "Expect '(' after {kind} name."
        )));
    };
    let mut params = vec![];
    if !(parser.check(&TokenType::RightParen)) {
        loop {
            if params.len() >= 255 {
                eprintln!("Can't have more than 255 parameters.");
            }
            let Ok(param) = parser.consume(&TokenType::Identifier) else {
                return Err(LoxError::SyntaxError("Expect parameter name.".into()));
            };
            params.push(param.clone());
            if !parser.matches(&[TokenType::Comma]) {
                break;
            }
        }
    }
    let Ok(_) = parser.consume(&TokenType::RightParen) else {
        return Err(LoxError::SyntaxError("Expect ')' after parameters.".into()));
    };
    let Ok(_) = parser.consume(&TokenType::LeftBrace) else {
        return Err(LoxError::SyntaxError(format!(
            "Expect '{{' before {kind} body."
        )));
    };
    let body = parse_block(parser)?;

    Ok(Stmt::Function {
        name: name.clone(),
        params,
        body: Box::new(body),
    })
}

// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
fn parse_var_declaration(parser: &mut Parser) -> Result<Stmt, LoxError> {
    let Ok(name_token) = parser.consume(&TokenType::Identifier) else {
        return Err(LoxError::SyntaxError("Expect variable name.".to_string()));
    };
    let name = name_token.lexeme.clone();

    let initializer = if parser.matches(&[TokenType::Equal]) {
        Some(parse_expression(parser)?)
    } else {
        None
    };

    let Ok(_) = parser.consume(&TokenType::Semicolon) else {
        return Err(LoxError::SyntaxError(
            "Expect ';' after variable declaration.".to_string(),
        ));
    };

    Ok(Stmt::Var { name, initializer })
}

// statement      → exprStmt
//                | forStmt
//                | ifStmt
//                | printStmt
//                | whileStmt
//                | block ;
fn parse_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    if parser.matches(&[TokenType::If]) {
        return parse_if_statement(parser);
    }
    if parser.matches(&[TokenType::For]) {
        return parse_for_statement(parser);
    }
    if parser.matches(&[TokenType::Print]) {
        return parse_print_statement(parser);
    }
    if parser.matches(&[TokenType::While]) {
        return parse_while_statement(parser);
    }
    if parser.matches(&[TokenType::LeftBrace]) {
        return parse_block(parser);
    }
    Ok(parse_expression_statement(parser))?
}

// ifStmt         → "if" "(" expression ")" statement
//                ( "else" statement )? ;
fn parse_if_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    if parser.consume(&TokenType::LeftParen).is_err() {
        return Err(LoxError::SyntaxError("Expect '(' after 'if'.".into()));
    }

    let condition = parse_expression(parser)?;

    if parser.consume(&TokenType::RightParen).is_err() {
        return Err(LoxError::SyntaxError("Expect ')' after condition.".into()));
    }

    let then_branch = Box::new(parse_statement(parser)?);

    let else_branch = if parser.matches(&[TokenType::Else]) {
        Some(Box::new(parse_statement(parser)?))
    } else {
        None
    };

    Ok(Stmt::If {
        condition,
        then_branch,
        else_branch,
    })
}

// forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
//                  expression? ";"
//                  expression? ")" statement ;
fn parse_for_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    if parser.consume(&TokenType::LeftParen).is_err() {
        return Err(LoxError::SyntaxError("Expect '(' after 'for'.".into()));
    }

    // Parse initializer
    let initializer = if parser.matches(&[TokenType::Semicolon]) {
        None
    } else if parser.matches(&[TokenType::Var]) {
        Some(parse_var_declaration(parser)?)
    } else {
        Some(parse_expression_statement(parser)?)
    };

    // Parse condition
    let condition = if parser.matches(&[TokenType::Semicolon]) {
        Expr::Literal(TokenValue::Boolean(true))
    } else {
        parse_expression(parser)?
    };
    if parser.consume(&TokenType::Semicolon).is_err() {
        return Err(LoxError::SyntaxError(
            "Expect ';' after loop condition.".into(),
        ));
    }

    // Parse increment
    let increment = if parser.check(&TokenType::RightParen) {
        None
    } else {
        Some(parse_expression(parser)?)
    };
    if parser.consume(&TokenType::RightParen).is_err() {
        return Err(LoxError::SyntaxError(
            "Expect ')' after for clauses.".into(),
        ));
    }

    // TODO: Parse body
    let for_body = parse_statement(parser)?;

    // Desugar into while loop
    let mut body_stmts = vec![for_body];
    if let Some(increment) = increment {
        body_stmts.push(Stmt::Expression(increment));
    }
    let mut body = Stmt::Block(body_stmts);

    // Add the condition
    body = Stmt::While {
        condition,
        body: Box::new(body),
    };

    // Add the initializer, if any
    if let Some(initializer) = initializer {
        body = Stmt::Block(vec![initializer, body]);
    }

    Ok(body)
}

fn parse_while_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    if parser.consume(&TokenType::LeftParen).is_err() {
        return Err(LoxError::SyntaxError("Expect '(' after 'while'.".into()));
    }
    let condition = parse_expression(parser)?;
    if parser.consume(&TokenType::RightParen).is_err() {
        return Err(LoxError::SyntaxError("Expect ')' after condition.".into()));
    }
    let body = parse_statement(parser)?;

    Ok(Stmt::While {
        condition,
        body: Box::new(body),
    })
}

// block          → "{" declaration* "}" ;
fn parse_block(parser: &mut Parser) -> Result<Stmt, LoxError> {
    let mut stmts = vec![];
    while !parser.check(&TokenType::RightBrace) && !parser.is_at_end() {
        stmts.push(parse_declaration(parser)?);
    }
    parser.consume(&TokenType::RightBrace)?;
    Ok(Stmt::Block(stmts))
}

// exprStmt       → expression ";" ;
fn parse_expression_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    let expression_statement = Stmt::Expression(parse_expression(parser)?);
    parser.consume(&TokenType::Semicolon)?;
    Ok(expression_statement)
}

// printStmt      → "print" expression ";" ;
fn parse_print_statement(parser: &mut Parser) -> Result<Stmt, LoxError> {
    let print_statement = Stmt::Print(parse_expression(parser)?);
    match parser.consume(&TokenType::Semicolon) {
        Ok(_) => Ok(print_statement),
        Err(e) => {
            eprintln!("[line 1] Expect semicolon");
            Err(e)
        }
    }
}

// expression     → assignment ;
fn parse_expression(parser: &mut Parser) -> Result<Expr, LoxError> {
    match parse_assignment(parser) {
        Ok(expression) => Ok(expression),
        Err(e) => {
            eprintln!("[line 1] Expected expression");
            Err(e)
        }
    }
}

// assignment     → IDENTIFIER "=" assignment
//                | logic_or ;
fn parse_assignment(parser: &mut Parser) -> Result<Expr, LoxError> {
    let expr = parse_or(parser)?;

    if parser.matches(&[TokenType::Equal]) {
        let equals = parser.previous();
        // Remember what the equals part is in case it needs to be returned in the error
        let equals_string = equals.to_string();
        let value = parse_assignment(parser)?;

        match expr {
            Expr::Variable(name) => {
                return Ok(Expr::Assign {
                    name: name.clone(),
                    value: Box::new(value),
                });
            }
            _ => {
                return Err(LoxError::SyntaxError(format!(
                    "Invalid assignment target {equals_string}"
                )));
            }
        }
    }

    Ok(expr)
}

// logic_or       → logic_and ( "or" logic_and )* ;
fn parse_or(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_and(parser)?;

    while parser.matches(&[TokenType::Or]) {
        let operator = parser.previous().clone();
        let right = parse_and(parser)?;
        expr = Expr::Logical {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

// logic_and      → equality ( "and" equality )* ;
fn parse_and(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_equality(parser)?;

    while parser.matches(&[TokenType::And]) {
        let operator = parser.previous().clone();
        let right = parse_equality(parser)?;
        expr = Expr::Logical {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
fn parse_equality(parser: &mut Parser) -> Result<Expr, LoxError> {
    let expr = parse_comparison(parser)?;

    if parser.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
        let operator = parser.previous().clone();
        let right = parse_comparison(parser)?;
        return Ok(Expr::Binary {
            operator,
            left: Box::new(expr),
            right: Box::new(right),
        });
    }

    Ok(expr)
}

// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
fn parse_comparison(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_term(parser)?;

    while parser.matches(&[
        TokenType::Greater,
        TokenType::GreaterEqual,
        TokenType::Less,
        TokenType::LessEqual,
    ]) {
        let operator = parser.previous().clone();
        let right = parse_term(parser)?;
        expr = Expr::Binary {
            operator,
            left: Box::new(expr),
            right: Box::new(right),
        };
    }

    Ok(expr)
}

// term           → factor ( ( "-" | "+" ) factor )* ;
fn parse_term(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_factor(parser)?;

    while parser.matches(&[TokenType::Minus, TokenType::Plus]) {
        let operator = parser.previous().clone();
        let right = parse_factor(parser)?;
        expr = Expr::Binary {
            operator,
            left: Box::new(expr),
            right: Box::new(right),
        };
    }

    Ok(expr)
}

// factor         → unary ( ( "/" | "*" ) unary )* ;
fn parse_factor(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_unary(parser)?;

    while parser.matches(&[TokenType::Slash, TokenType::Star]) {
        let operator = parser.previous().clone();
        let right = parse_unary(parser)?;
        expr = Expr::Binary {
            operator,
            left: Box::new(expr),
            right: Box::new(right),
        };
    }

    Ok(expr)
}

// unary          → ( "!" | "-" ) unary | call;
fn parse_unary(parser: &mut Parser) -> Result<Expr, LoxError> {
    if parser.matches(&[TokenType::Bang, TokenType::Minus]) {
        let operator = parser.previous().clone();
        let right = parse_unary(parser)?;
        return Ok(Expr::Unary {
            operator,
            right: Box::new(right),
        });
    }

    parse_call(parser)
}

// call           → primary ( "(" arguments? ")" )* ;
fn parse_call(parser: &mut Parser) -> Result<Expr, LoxError> {
    let mut expr = parse_primary(parser)?;

    loop {
        if parser.matches(&[TokenType::LeftParen]) {
            expr = finish_call(parser, expr)?;
        } else {
            break;
        }
    }
    Ok(expr)
}

// Helper to finish the call parsing
fn finish_call(parser: &mut Parser, callee: Expr) -> Result<Expr, LoxError> {
    let mut arguments = vec![];
    if !parser.check(&TokenType::RightParen) {
        loop {
            if arguments.len() >= 255 {
                eprintln!("Can't have more than 255 arguments.");
            }
            arguments.push(parse_expression(parser)?);
            if !parser.matches(&[TokenType::Comma]) {
                break;
            }
        }
    }

    let Ok(paren) = parser.consume(&TokenType::RightParen) else {
        return Err(LoxError::SyntaxError(
            "Expecting ')' after function arguments".to_string(),
        ));
    };

    Ok(Expr::Call {
        callee: Box::new(callee),
        paren: paren.clone(),
        arguments,
    })
}

// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;
fn parse_primary(parser: &mut Parser) -> Result<Expr, LoxError> {
    if parser.matches(&[TokenType::Number]) {
        let n = parser
            .previous()
            .literal
            .clone()
            .expect("Number token without literal must not exist");
        return Ok(Expr::Literal(n));
    }

    if parser.matches(&[TokenType::String]) {
        let s = parser
            .previous()
            .literal
            .clone()
            .expect("String token without literal must not exist");
        return Ok(Expr::Literal(s));
    }

    if parser.matches(&[TokenType::Nil]) {
        return Ok(Expr::Literal(TokenValue::Nil));
    }

    if parser.matches(&[TokenType::True]) {
        return Ok(Expr::Literal(TokenValue::Boolean(true)));
    }

    if parser.matches(&[TokenType::False]) {
        return Ok(Expr::Literal(TokenValue::Boolean(false)));
    }

    if parser.matches(&[TokenType::LeftParen]) {
        let expr = parse_expression(parser)?;
        if parser.consume(&TokenType::RightParen).is_ok() {
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        eprintln!("[line 1] Missing closing parenthesis.");
        return Err(LoxError::SyntaxError(
            "Missing closing parenthesis".to_string(),
        ));
    }

    if parser.matches(&[TokenType::Identifier]) {
        return Ok(Expr::Variable(parser.previous().lexeme.clone()));
    }

    Err(LoxError::SyntaxError(
        "No matching primary found".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_assignment() {
        let var_a = Token {
            token_type: TokenType::Identifier,
            lexeme: "a".to_string(),
            literal: None,
        };
        let equal = Token {
            token_type: TokenType::Equal,
            lexeme: "=".to_string(),
            literal: None,
        };
        let fourtytwo = Token {
            token_type: TokenType::Number,
            lexeme: "42.0".to_string(),
            literal: Some(TokenValue::Number(42.0)),
        };

        let mut parser = Parser::new(vec![var_a.clone(), equal, fourtytwo, Token::eof()]);
        let assignment = parse_assignment(&mut parser).unwrap();

        assert_eq!(
            assignment,
            Expr::Assign {
                name: var_a.lexeme,
                value: Box::new(Expr::Literal(TokenValue::Number(42.0)))
            }
        );
    }

    #[test]
    fn test_parse_primary_nil() {
        let token = Token {
            token_type: TokenType::Nil,
            lexeme: "nil".to_string(),
            literal: None,
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "nil".to_string());
    }

    #[test]
    fn test_parse_primary_number() {
        let token = Token {
            token_type: TokenType::Number,
            lexeme: String::new(),
            literal: Some(TokenValue::Number(42.0)),
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "42.0".to_string());
    }

    #[test]
    fn test_parse_primary_string() {
        let token = Token {
            token_type: TokenType::String,
            lexeme: String::new(),
            literal: Some(TokenValue::String("42".to_string())),
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "42".to_string());
    }

    #[test]
    fn test_parse_primary_grouping() {
        let left_paren = Token {
            token_type: TokenType::LeftParen,
            lexeme: "(".to_string(),
            literal: None,
        };
        let token = Token {
            token_type: TokenType::String,
            lexeme: String::new(),
            literal: Some(TokenValue::String("foo".to_string())),
        };
        let right_paren = Token {
            token_type: TokenType::RightParen,
            lexeme: ")".to_string(),
            literal: None,
        };
        let tokens = vec![
            left_paren,
            token,
            right_paren,
            Token::semicolon(),
            Token::eof(),
        ];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary[0].to_string(), "(group foo)".to_string());
    }

    #[test]
    fn test_parse_unary() {
        let bang = Token {
            token_type: TokenType::Bang,
            lexeme: "!".to_string(),
            literal: None,
        };
        let token_true = Token {
            token_type: TokenType::True,
            lexeme: String::new(),
            literal: None,
        };
        let tokens = vec![
            bang.clone(),
            bang,
            token_true,
            Token::semicolon(),
            Token::eof(),
        ];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary[0].to_string(), "(! (! true))".to_string());
    }

    #[test]
    fn test_parse_binary() {
        let token_left = Token {
            token_type: TokenType::Number,
            lexeme: "40".to_string(),
            literal: Some(TokenValue::Number(40.0)),
        };
        let plus = Token {
            token_type: TokenType::Plus,
            lexeme: "+".to_string(),
            literal: None,
        };
        let token_right = Token {
            token_type: TokenType::Number,
            lexeme: "2".to_string(),
            literal: Some(TokenValue::Number(2.0)),
        };
        let tokens = vec![
            token_left,
            plus,
            token_right,
            Token::semicolon(),
            Token::eof(),
        ];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary[0].to_string(), "(+ 40.0 2.0)".to_string());
    }
}
