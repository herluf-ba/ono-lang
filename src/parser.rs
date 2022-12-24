use crate::ast::*;
use crate::error::Error;
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

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Expr, Error> {
        self.tokens = tokens;
        let result = self.expression();
        self.tokens.clear();
        result
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

    // expression -> equality
    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    // equality -> comparison ( ("!=" | "==") comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = match self.comparison() {
            Ok(expr) => expr,
            Err(error) => return Err(error),
        };
        while self.is_token_of_kind(&[TokenKind::BANGEQUAL, TokenKind::EQUALEQUAL]) {
            let right = match self.comparison() {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            };
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // comparison -> term ( (">" | ">=" | "<" | "<=") term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = match self.term() {
            Ok(expr) => expr,
            Err(error) => return Err(error),
        };
        while self.is_token_of_kind(&[
            TokenKind::LESS,
            TokenKind::LESSEQUAL,
            TokenKind::GREATER,
            TokenKind::GREATEREQUAL,
        ]) {
            let right = match self.term() {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            };
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // term -> factor ( ("-" | "+") factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = match self.factor() {
            Ok(expr) => expr,
            Err(error) => return Err(error),
        };
        while self.is_token_of_kind(&[TokenKind::MINUS, TokenKind::PLUS]) {
            let right = match self.factor() {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            };
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // factor -> unary ( ("/" | "*") unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = match self.unary() {
            Ok(expr) => expr,
            Err(error) => return Err(error),
        };

        while self.is_token_of_kind(&[TokenKind::SLASH, TokenKind::STAR]) {
            let right = match self.unary() {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            };
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // unary -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            let inner = match self.unary() {
                Ok(inner) => inner,
                Err(error) => return Err(error),
            };
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(inner),
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

        if self.is_token_of_kind(&[TokenKind::LEFTPAREN]) {
            let expr = match self.expression() {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            };

            if self.consume(&TokenKind::RIGHTPAREN).is_none() {
                //self.errors.push(
                //self.error_producer
                //.syntax_error_from_token(self.previous(), "Expected closing ')'"),
                //);
            }

            return Ok(Expr::Group {
                expr: Box::new(expr),
            });
        }
        //self.error_producer
        //.syntax_error_from_token(self.previous(), "Expected expression"),

        // TODO: return Err here
        panic!("Parser matching bottomed out. This is an ono implementation error")
    }
}
