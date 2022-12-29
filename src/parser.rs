use crate::ast::*;
use crate::error::{Error, SyntaxError};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// A recursive descent parser that produces an AST
impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
        }
    }

    pub fn reset(&mut self) {
        self.tokens.clear();
        self.current = 0;
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<Stmt>, Vec<Error>> {
        self.tokens = tokens;
        let mut errors = Vec::new();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            };
        }
        self.reset();

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(statements)
        }
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn advance(&mut self) -> &Token {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, kind: &TokenKind) -> Option<&Token> {
        if self.check(kind) {
            return Some(self.advance());
        }
        None
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn check(&mut self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind.is_same(kind)
    }

    fn is_token_of_kind(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn synchronize(&mut self) {
        // Skip ahead until the start of the next statement is found
        while !self.is_at_end() {
            if self.previous().kind.is_same(&TokenKind::SEMICOLON) {
                return;
            }

            match self.peek().kind {
                TokenKind::CLASS
                | TokenKind::FUN
                | TokenKind::LET
                | TokenKind::FOR
                | TokenKind::IF
                | TokenKind::WHILE
                | TokenKind::PRINT
                | TokenKind::RETURN => {
                    return;
                }
                _ => self.advance(),
            };
        }
    }

    // declaration -> let_declaration | statement ;
    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.is_token_of_kind(&[TokenKind::LET]) {
            return self.let_declaration();
        }

        self.statement()
    }

    // let_declaration -> "let" IDENTIFIER ( "=" expression )? ";" ;
    fn let_declaration(&mut self) -> Result<Stmt, Error> {
        let name = match self.consume(&TokenKind::IDENTIFIER("".to_string())) {
            Some(token) => token.clone(),
            None => return Err(Error::syntax_error(SyntaxError::S004, self.peek().clone())),
        };

        let mut initializer = Expr::Literal {
            value: Token {
                kind: TokenKind::NULL,
                lexeme: "".to_string(),
                row: 0,
                column: 0,
            },
        };

        if self.is_token_of_kind(&[TokenKind::EQUAL]) {
            initializer = self.expression()?;
        }

        match self.consume(&TokenKind::SEMICOLON) {
            Some(_) => Ok(Stmt::Let { name, initializer }),
            None => Err(Error::syntax_error(SyntaxError::S005, self.peek().clone())),
        }
    }

    // statement -> expression_statement | print_statement | block ;
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.is_token_of_kind(&[TokenKind::PRINT]) {
            return self.print_statement();
        }

        if self.is_token_of_kind(&[TokenKind::LEFTBRACE]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        self.expression_statement()
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let opening_brace = self.peek().clone();
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RIGHTBRACE) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        match self.consume(&TokenKind::RIGHTBRACE) {
            Some(_) => Ok(statements),
            None => Err(Error::syntax_error(SyntaxError::S006, opening_brace)),
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        if self.consume(&TokenKind::SEMICOLON).is_none() {
            return Err(Error::syntax_error(SyntaxError::S005, self.peek().clone()));
        }

        Ok(Stmt::Expression { expr })
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        if self.consume(&TokenKind::SEMICOLON).is_none() {
            return Err(Error::syntax_error(SyntaxError::S005, self.peek().clone()));
        }

        Ok(Stmt::Print { expr })
    }
    // expression -> assigment
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assigment()
    }

    // assigment -> IDENTIFIER "=" assigment | equality ;
    fn assigment(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;
        if self.is_token_of_kind(&[TokenKind::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assigment()?;
            return match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    expr: Box::new(value),
                }),
                _ => Err(Error::syntax_error(SyntaxError::S007, equals)),
            };
        }

        Ok(expr)
    }

    // equality -> comparison ( ("!=" | "==") comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        while self.is_token_of_kind(&[TokenKind::BANGEQUAL, TokenKind::EQUALEQUAL]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.comparison()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // comparison -> term ( (">" | ">=" | "<" | "<=") term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;
        while self.is_token_of_kind(&[
            TokenKind::LESS,
            TokenKind::LESSEQUAL,
            TokenKind::GREATER,
            TokenKind::GREATEREQUAL,
        ]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.term()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // term -> factor ( ("-" | "+") factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        while self.is_token_of_kind(&[TokenKind::MINUS, TokenKind::PLUS]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.factor()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // factor -> unary ( ("/" | "*") unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.is_token_of_kind(&[TokenKind::SLASH, TokenKind::STAR]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // unary -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(self.unary()?),
            });
        }
        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "null" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[
            TokenKind::FALSE,
            TokenKind::TRUE,
            TokenKind::NULL,
            TokenKind::NUMBER(1.0),
            TokenKind::STRING("".to_string()),
        ]) {
            return Ok(Expr::Literal {
                value: self.previous().clone(),
            });
        }

        if self.is_token_of_kind(&[TokenKind::IDENTIFIER("".to_string())]) {
            return Ok(Expr::Variable {
                name: self.previous().clone(),
            });
        }

        if self.is_token_of_kind(&[TokenKind::LEFTPAREN]) {
            let opening_token = self.previous().clone();
            let expr = self.expression()?;

            return if self.consume(&TokenKind::RIGHTPAREN).is_none() {
                Err(Error::syntax_error(SyntaxError::S006, opening_token))
            } else {
                Ok(Expr::Group {
                    expr: Box::new(expr),
                })
            };
        }

        Err(Error::syntax_error(SyntaxError::S003, self.peek().clone()))
    }
}
