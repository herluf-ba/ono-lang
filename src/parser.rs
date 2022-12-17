use crate::ast::*;
use crate::error::{Error, ErrorKind, ErrorReporter};
use crate::lexer::{Lexer, Token, TokenKind};

pub struct AstBuilder<'a> {
    lexer: &'a Lexer<'a>,
    errors: Vec<Error>,
    current: usize,
}

impl<'a> ErrorReporter for AstBuilder<'a> {
    fn add_error(&mut self, error: Error) {
        self.errors.push(error)
    }

    fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

// A recursive descent parser that takes a lexer and produces an AST
impl<'a> AstBuilder<'a> {
    pub fn from(lexer: &'a Lexer) -> Self {
        Self {
            lexer,
            errors: Vec::new(),
            current: 0,
        }
    }

    pub fn build(&mut self) -> Option<Expr> {
        let ast = self.expression();
        if self.is_ok() {
            return Some(ast);
        }
        None
    }

    fn add_syntax_error(&mut self, message: &str) {
        let token = self.peek();
        self.errors.push(Error::new(
            ErrorKind::SyntaxError,
            Some(token.row),
            Some(token.column),
            self.lexer.lines[token.row],
            message,
        ))
    }

    fn previous(&self) -> &Token {
        self.lexer.tokens.get(self.current - 1).unwrap()
    }

    fn peek(&self) -> &Token {
        self.lexer.tokens.get(self.current).unwrap()
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

    // expression -> equality
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    // equality -> comparison ( ("!=" | "==") comparison )* ;
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.is_token_of_kind(&[TokenKind::BANGEQUAL, TokenKind::EQUALEQUAL]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.comparison()),
                left: Box::new(expr),
            }
        }
        expr
    }

    // comparison -> term ( (">" | ">=" | "<" | "<=") term )* ;
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.is_token_of_kind(&[
            TokenKind::LESS,
            TokenKind::LESSEQUAL,
            TokenKind::GREATER,
            TokenKind::GREATEREQUAL,
        ]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.term()),
                left: Box::new(expr),
            }
        }
        expr
    }

    // term -> factor ( ("-" | "+") factor )* ;
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.is_token_of_kind(&[TokenKind::MINUS, TokenKind::PLUS]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.factor()),
                left: Box::new(expr),
            }
        }
        expr
    }

    // factor -> unary ( ("/" | "*") unary )* ;
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.is_token_of_kind(&[TokenKind::SLASH, TokenKind::STAR]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()),
                left: Box::new(expr),
            }
        }
        expr
    }

    // unary -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> Expr {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            return Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(self.unary()),
            };
        }
        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "null" | "(" expression ")" ;
    fn primary(&mut self) -> Expr {
        if self.is_token_of_kind(&[
            TokenKind::FALSE,
            TokenKind::TRUE,
            TokenKind::NULL,
            TokenKind::NUMBER(1.0),
            TokenKind::STRING("".to_string()),
        ]) {
            return Expr::Literal {
                value: self.previous().clone(),
            };
        }

        if self.is_token_of_kind(&[TokenKind::LEFTPAREN]) {
            let expr = self.expression();
            if self.consume(&TokenKind::RIGHTPAREN).is_none() {
                self.add_syntax_error("Expected closing ')'");
            }

            return Expr::Group {
                expr: Box::new(expr),
            };
        }

        self.add_syntax_error("Expected expression");

        Expr::Error {}
    }
}
