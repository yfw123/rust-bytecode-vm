use std::iter::Peekable;
use std::str::Chars;

use crate::agent::Agent;

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenType {
    Identifier,
    Integer,
    Double,
    String,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,
    And,
    Pipe,
    Caret,
    AndAnd,
    PipePipe,
    Semicolon,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Comma,

    Return,
    Function,
    For,
    If,
    Else,
    While,
    Break,
    Continue,
    Let,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Position {
    line: usize,
    column: usize,
}

#[derive(Debug, PartialEq, Clone)]
struct Token {
    position: Position,
    typ: TokenType,
    text: String,
}

impl Token {
    pub fn new<T>(typ: TokenType, line: usize, column: usize, text: T) -> Token
    where
        T: Into<String>,
    {
        Token {
            position: Position { line, column },
            typ,
            text: text.into(),
        }
    }
}

struct Lexer<'a> {
    position: usize,
    line: usize,
    column: usize,
    input: &'a str,
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            position: 0,
            line: 1,
            column: 1,
            input,
            chars: input.chars().peekable(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.chars.next();
        if next.is_some() {
            self.column += 1;
            self.position += 1;
        }
        next
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    pub fn next_token(&mut self) -> Option<Result<Token, String>> {
        let mut start = self.position;
        let mut start_line = self.line;
        let mut start_column = self.column;

        macro_rules! token {
            ($typ:expr) => {{
                return Some(Ok(Token::new(
                    $typ,
                    start_line,
                    start_column,
                    &self.input[start..self.position],
                )));
            }};
        }

        macro_rules! error {
            ($message:expr $(, $stuff:expr)* $(,)?) => {
                Some(Err(format!($message, $($stuff)*)))
            };
        }

        while let Some(c) = self.next_char() {
            macro_rules! or2 {
                ($char:expr, $typ:expr, $typ2:expr) => {{
                    if let Some(c) = self.peek_char() {
                        if *c == $char {
                            self.next_char();
                            token!($typ2);
                        }
                    }
                    token!($typ);
                }};
            }

            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    start_line += 1;
                    start_column = 1;
                    start += 1;
                }
                '\r' | ' ' | '\t' => {
                    start += 1;
                    start_column += 1;
                }

                '#' => {
                    while let Some(c) = self.next_char() {
                        if c == '\n' {
                            break;
                        }
                    }
                    start += 1;
                }

                '+' => token!(TokenType::Plus),
                '-' => token!(TokenType::Minus),
                '/' => token!(TokenType::Slash),
                '^' => token!(TokenType::Caret),
                '[' => token!(TokenType::LeftBracket),
                ']' => token!(TokenType::RightBracket),
                '{' => token!(TokenType::LeftBrace),
                '}' => token!(TokenType::RightBrace),
                '(' => token!(TokenType::LeftParen),
                ')' => token!(TokenType::RightParen),
                '%' => token!(TokenType::Percent),
                ';' => token!(TokenType::Semicolon),
                ',' => token!(TokenType::Comma),
                '*' => or2!('*', TokenType::Star, TokenType::StarStar),
                '&' => or2!('&', TokenType::And, TokenType::AndAnd),
                '|' => or2!('|', TokenType::Pipe, TokenType::PipePipe),
                '=' => or2!('=', TokenType::Equal, TokenType::EqualEqual),
                '<' => or2!('=', TokenType::LessThan, TokenType::LessThanEqual),
                '>' => or2!('=', TokenType::GreaterThan, TokenType::GreaterThanEqual),
                '!' => or2!('=', TokenType::Bang, TokenType::BangEqual),

                '0'..='9' => {
                    let mut is_double = false;
                    while let Some(c) = self.peek_char() {
                        match c {
                            '0'..='9' => {
                                self.next_char();
                            }
                            '.' => {
                                self.next_char();
                                if is_double {
                                    return error!("Unexpected '.'");
                                } else {
                                    is_double = true;
                                }
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    token!(if is_double {
                        TokenType::Double
                    } else {
                        TokenType::Integer
                    });
                }

                'a'..='z' | 'A'..='Z' | '_' => {
                    while let Some(c) = self.peek_char() {
                        match c {
                            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                                self.next_char();
                            }
                            _ => break,
                        }
                    }

                    match &self.input[start..self.position] {
                        "return" => token!(TokenType::Return),
                        "for" => token!(TokenType::For),
                        "while" => token!(TokenType::While),
                        "function" => token!(TokenType::Function),
                        "let" => token!(TokenType::Let),
                        "if" => token!(TokenType::If),
                        "else" => token!(TokenType::Else),
                        "break" => token!(TokenType::Break),
                        "continue" => token!(TokenType::Continue),
                        _ => token!(TokenType::Identifier),
                    }
                }

                '"' => {
                    while let Some(c) = self.peek_char() {
                        match c {
                            '\\' => {
                                self.next_char();
                                if let Some(c) = self.next_char() {
                                    match c {
                                        'n' | 't' | '"' => {}
                                        _ => return error!("unrecognized escape sequence"),
                                    }
                                } else {
                                    return error!("unterminated escape sequence");
                                }
                            }
                            '"' => {
                                self.next_char();
                                break;
                            }
                            _ => {
                                self.next_char();
                            }
                        }
                    }

                    token!(TokenType::String);
                }

                _ => return error!("Unexpected character '{}'", c),
            }
        }

        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Statement {
    position: Position,
    value: StatementKind,
}

#[derive(Debug, PartialEq, Clone)]
enum StatementKind {
    FunctionDeclaration {
        name: Expression,
        parameters: Vec<Expression>,
        body: Vec<Statement>,
    },
    LetDeclaration {
        name: Expression,
        value: Option<Expression>,
    },
    IfStatement {
        predicate: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    WhileStatement {
        predicate: Expression,
        body: Vec<Statement>,
    },
    ForStatement {
        initializer: Option<Box<Statement>>,
        predicate: Option<Expression>,
        increment: Option<Expression>,
        body: Vec<Statement>,
    },
    BreakStatement,
    ContinueStatement,
    ReturnStatement(Option<Expression>),
    ExpressionStatement(Expression),
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,
    And,
    Pipe,
    Caret,
    AndAnd,
    PipePipe,
}

#[derive(Debug, PartialEq, Clone)]
struct Expression {
    position: Position,
    value: ExpressionKind,
}

#[derive(Debug, PartialEq, Clone)]
enum ExpressionKind {
    Identifier(usize),
    Integer(i64),
    Double(f64),
    Function {
        name: Option<usize>,
        parameters: Vec<Expression>,
        body: Vec<Statement>,
    },
    UnaryOperation(Operator, Box<Expression>),
    BinaryOperation(Box<Expression>, Operator, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
}

pub type ParseResult<T> = Result<T, String>;

struct Parser<'a> {
    agent: &'a mut Agent,
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(agent: &'a mut Agent, lexer: Lexer<'a>) -> Parser<'a> {
        Parser {
            agent,
            lexer: lexer.peekable(),
        }
    }

    pub fn next_statement(&mut self) -> Option<ParseResult<Statement>> {
        match self.peek() {
            Ok(Some(_)) => Some(self.parse_statement()),
            Err(msg) => Some(Err(msg.clone())),
            Ok(None) => None,
        }
    }

    fn next_token(&mut self) -> ParseResult<Option<Token>> {
        self.lexer.next().transpose()
    }

    fn expect(&mut self, expected: TokenType) -> ParseResult<Token> {
        match &self.next_token()? {
            Some(tok) if tok.typ == expected => Ok(tok.clone()),
            Some(Token {
                typ,
                position: Position { line, column },
                ..
            }) => Err(format!(
                "Expected {:?}, got {:?} at {}:{}",
                expected, typ, line, column
            )),
            None => Err(format!("Expected {:?}, found end of input", expected)),
        }
    }

    fn peek(&mut self) -> ParseResult<Option<&Token>> {
        match self.lexer.peek() {
            Some(&Ok(ref tok)) => Ok(Some(tok)),
            Some(Err(msg)) => Err(msg.clone()),
            None => Ok(None),
        }
    }

    fn matches(&mut self, expected: TokenType) -> ParseResult<bool> {
        match self.peek()? {
            Some(&Token { typ, .. }) if typ == expected => {
                self.expect(expected)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        if let Some(token) = self.peek()? {
            match token.typ {
                TokenType::Let => self.parse_let_declaration(),
                TokenType::Function => self.parse_function_declaration(),
                TokenType::If => self.parse_if_statement(),
                TokenType::While => self.parse_while_statement(),
                TokenType::For => self.parse_for_statement(),
                TokenType::Continue => self.parse_continue_statement(),
                TokenType::Break => self.parse_break_statement(),
                TokenType::Return => self.parse_return_statement(),
                _ => self.parse_expression_statement(),
            }
        } else {
            unreachable!();
        }
    }

    fn parse_let_declaration(&mut self) -> ParseResult<Statement> {
        let let_ = self.expect(TokenType::Let)?;
        let ident = self.parse_identifier_expression()?;

        let value = if self.matches(TokenType::Equal)? {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(TokenType::Semicolon)?;

        Ok(Statement {
            position: let_.position,
            value: StatementKind::LetDeclaration { name: ident, value },
        })
    }

    fn parse_function_declaration(&mut self) -> ParseResult<Statement> {
        let function = self.expect(TokenType::Function)?;
        let ident = self.parse_expression()?;

        if let ExpressionKind::Identifier(_) = ident.value {
        } else {
            return Err(format!(
                "Expected identifier at {}:{}",
                ident.position.line, ident.position.column
            ));
        }

        self.expect(TokenType::LeftParen)?;

        let mut params = Vec::new();
        while !self.matches(TokenType::RightParen)? {
            let param = self.parse_expression()?;
            if let ExpressionKind::Identifier(_) = param.value {
                params.push(param);
            } else {
                return Err(format!(
                    "Expected identifier at {}:{}",
                    param.position.line, param.position.column
                ));
            }
            if !self.matches(TokenType::Comma)? {
                self.expect(TokenType::RightParen)?;
                break;
            }
        }

        let mut body = Vec::new();

        self.expect(TokenType::LeftBrace)?;
        while !self.matches(TokenType::RightBrace)? {
            body.push(self.parse_statement()?);
        }

        Ok(Statement {
            position: function.position,
            value: StatementKind::FunctionDeclaration {
                name: ident,
                parameters: params,
                body,
            },
        })
    }

    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        let if_ = self.expect(TokenType::If)?;
        let predicate = self.parse_expression()?;
        self.expect(TokenType::LeftBrace)?;

        let mut then_body = Vec::new();
        while !self.matches(TokenType::RightBrace)? {
            then_body.push(self.parse_statement()?);
        }

        let else_body = if self.matches(TokenType::Else)? {
            let peeked = self.peek()?;
            if peeked.is_some() && peeked.unwrap().typ == TokenType::If {
                Some(vec![self.parse_if_statement()?])
            } else {
                self.expect(TokenType::LeftBrace)?;
                let mut stmts = Vec::new();
                while !self.matches(TokenType::RightBrace)? {
                    stmts.push(self.parse_statement()?);
                }
                Some(stmts)
            }
        } else {
            None
        };

        Ok(Statement {
            position: if_.position,
            value: StatementKind::IfStatement {
                predicate,
                then_body,
                else_body,
            },
        })
    }

    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        let while_ = self.expect(TokenType::While)?;
        let predicate = self.parse_expression()?;

        self.expect(TokenType::LeftBrace)?;

        let mut body = Vec::new();
        while !self.matches(TokenType::RightBrace)? {
            body.push(self.parse_statement()?);
        }

        Ok(Statement {
            position: while_.position,
            value: StatementKind::WhileStatement { predicate, body },
        })
    }

    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        let for_ = self.expect(TokenType::For)?;

        let initializer = if self.matches(TokenType::Semicolon)? {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };

        let predicate = if self.matches(TokenType::Semicolon)? {
            None
        } else {
            let pred = self.parse_expression()?;
            self.expect(TokenType::Semicolon)?;
            Some(pred)
        };

        let increment = if self.matches(TokenType::LeftBrace)? {
            None
        } else {
            let inc = self.parse_expression()?;
            self.expect(TokenType::LeftBrace)?;
            Some(inc)
        };

        let mut body = Vec::new();
        while !self.matches(TokenType::RightBrace)? {
            body.push(self.parse_statement()?);
        }

        Ok(Statement {
            position: for_.position,
            value: StatementKind::ForStatement {
                initializer,
                predicate,
                increment,
                body,
            },
        })
    }

    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        let return_ = self.expect(TokenType::Return)?;

        let expression = if self.matches(TokenType::Semicolon)? {
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenType::Semicolon)?;
            Some(expr)
        };

        Ok(Statement {
            position: return_.position,
            value: StatementKind::ReturnStatement(expression),
        })
    }

    fn parse_continue_statement(&mut self) -> ParseResult<Statement> {
        let continue_ = self.expect(TokenType::Continue)?;
        self.expect(TokenType::Semicolon)?;
        Ok(Statement {
            position: continue_.position,
            value: StatementKind::ContinueStatement,
        })
    }

    fn parse_break_statement(&mut self) -> ParseResult<Statement> {
        let break_ = self.expect(TokenType::Break)?;
        self.expect(TokenType::Semicolon)?;
        Ok(Statement {
            position: break_.position,
            value: StatementKind::BreakStatement,
        })
    }

    fn parse_expression_statement(&mut self) -> ParseResult<Statement> {
        let expression = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;

        Ok(Statement {
            position: expression.position,
            value: StatementKind::ExpressionStatement(expression),
        })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        if let Some(token) = self.peek()? {
            match token.typ {
                TokenType::Identifier => self.parse_identifier_expression(),
                TokenType::Integer => self.parse_integer_expression(),
                TokenType::Double => self.parse_double_expression(),
                _ => unimplemented!(),
            }
        } else {
            unreachable!();
        }
    }

    fn parse_identifier_expression(&mut self) -> ParseResult<Expression> {
        let Token {
            typ,
            position,
            text,
        } = self.expect(TokenType::Identifier)?;
        if typ == TokenType::Identifier {
            let s = text.to_string();
            let id = self.agent.intern_string(s.as_ref());
            Ok(Expression {
                position,
                value: ExpressionKind::Identifier(id),
            })
        } else {
            unreachable!();
        }
    }

    fn parse_integer_expression(&mut self) -> ParseResult<Expression> {
        let number = self.expect(TokenType::Integer)?;

        let intval = number
            .text
            .parse()
            .expect("Failed to parse lexed integer literal");

        Ok(Expression {
            position: number.position,
            value: ExpressionKind::Integer(intval),
        })
    }

    fn parse_double_expression(&mut self) -> ParseResult<Expression> {
        let number = self.expect(TokenType::Double)?;

        let floatval: f64 = number
            .text
            .parse()
            .expect("Failed to parse lexed integer literal");

        Ok(Expression {
            position: number.position,
            value: ExpressionKind::Double(floatval),
        })
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = ParseResult<Statement>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_statement()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lexer_symbols() {
        let input = "[ ] { } ( ) + - * ** / & && | || ^ % ; < > <= >= = == ! != ,";
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![
                Token::new(TokenType::LeftBracket, 1, 1, "["),
                Token::new(TokenType::RightBracket, 1, 3, "]"),
                Token::new(TokenType::LeftBrace, 1, 5, "{"),
                Token::new(TokenType::RightBrace, 1, 7, "}"),
                Token::new(TokenType::LeftParen, 1, 9, "("),
                Token::new(TokenType::RightParen, 1, 11, ")"),
                Token::new(TokenType::Plus, 1, 13, "+"),
                Token::new(TokenType::Minus, 1, 15, "-"),
                Token::new(TokenType::Star, 1, 17, "*"),
                Token::new(TokenType::StarStar, 1, 19, "**"),
                Token::new(TokenType::Slash, 1, 22, "/"),
                Token::new(TokenType::And, 1, 24, "&"),
                Token::new(TokenType::AndAnd, 1, 26, "&&"),
                Token::new(TokenType::Pipe, 1, 29, "|"),
                Token::new(TokenType::PipePipe, 1, 31, "||"),
                Token::new(TokenType::Caret, 1, 34, "^"),
                Token::new(TokenType::Percent, 1, 36, "%"),
                Token::new(TokenType::Semicolon, 1, 38, ";"),
                Token::new(TokenType::LessThan, 1, 40, "<"),
                Token::new(TokenType::GreaterThan, 1, 42, ">"),
                Token::new(TokenType::LessThanEqual, 1, 44, "<="),
                Token::new(TokenType::GreaterThanEqual, 1, 47, ">="),
                Token::new(TokenType::Equal, 1, 50, "="),
                Token::new(TokenType::EqualEqual, 1, 52, "=="),
                Token::new(TokenType::Bang, 1, 55, "!"),
                Token::new(TokenType::BangEqual, 1, 57, "!="),
                Token::new(TokenType::Comma, 1, 60, ","),
            ],
        );
    }

    #[test]
    fn test_integer() {
        let input = "123";
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![Token::new(TokenType::Integer, 1, 1, "123")]
        );
    }

    #[test]
    fn test_double() {
        let input = "1.23";
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![Token::new(TokenType::Double, 1, 1, "1.23")]
        );
    }

    #[test]
    fn test_identifier() {
        let input = "
abc
_a
_
_1312
return
while
for
let
function
";
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![
                Token::new(TokenType::Identifier, 2, 1, "abc"),
                Token::new(TokenType::Identifier, 3, 1, "_a"),
                Token::new(TokenType::Identifier, 4, 1, "_"),
                Token::new(TokenType::Identifier, 5, 1, "_1312"),
                Token::new(TokenType::Return, 6, 1, "return"),
                Token::new(TokenType::While, 7, 1, "while"),
                Token::new(TokenType::For, 8, 1, "for"),
                Token::new(TokenType::Let, 9, 1, "let"),
                Token::new(TokenType::Function, 10, 1, "function"),
            ],
        );
    }

    #[test]
    fn test_string() {
        let input = r#""this is a string""#;
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![Token::new(TokenType::String, 1, 1, r#""this is a string""#),]
        );
    }

    #[test]
    fn test_string_escapes() {
        let input = r#""so I says, \"this\nis\tan escaped string\"""#;
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.filter_map(|a| a.ok()).collect::<Vec<_>>(),
            vec![Token::new(
                TokenType::String,
                1,
                1,
                r#""so I says, \"this\nis\tan escaped string\"""#
            ),]
        );
    }

    #[test]
    fn test_lexing_language_snippet() {
        let input = r#"
            function testing() {
                return "hello world";
            }

            print(testing());
        "#;
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer
                .filter_map(|tok| tok.ok().map(|tok| (tok.typ, tok.text)))
                .collect::<Vec<_>>(),
            vec![
                (TokenType::Function, "function".to_string()),
                (TokenType::Identifier, "testing".to_string()),
                (TokenType::LeftParen, "(".to_string()),
                (TokenType::RightParen, ")".to_string()),
                (TokenType::LeftBrace, "{".to_string()),
                (TokenType::Return, "return".to_string()),
                (TokenType::String, r#""hello world""#.to_string()),
                (TokenType::Semicolon, ";".to_string()),
                (TokenType::RightBrace, "}".to_string()),
                (TokenType::Identifier, "print".to_string()),
                (TokenType::LeftParen, "(".to_string()),
                (TokenType::Identifier, "testing".to_string()),
                (TokenType::LeftParen, "(".to_string()),
                (TokenType::RightParen, ")".to_string()),
                (TokenType::RightParen, ")".to_string()),
                (TokenType::Semicolon, ";".to_string()),
            ],
        );
    }

    #[test]
    fn test_lexer_iterator() {
        let input = "test 1 1.2";
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer
                .filter_map(|t| t.ok().map(|t| (t.typ, t.text)))
                .collect::<Vec<_>>(),
            vec![
                (TokenType::Identifier, "test".to_string()),
                (TokenType::Integer, "1".to_string()),
                (TokenType::Double, "1.2".to_string()),
            ]
        );
    }

    #[test]
    fn test_let_declaration() {
        let mut agent = Agent::new();
        let name = agent.intern_string("something");
        let someident = agent.intern_string("someident");
        let input = "let something = someident;";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.filter_map(|s| s.ok()).collect::<Vec<_>>(),
            vec![Statement {
                position: Position { line: 1, column: 1 },
                value: StatementKind::LetDeclaration {
                    name: Expression {
                        position: Position { line: 1, column: 5 },
                        value: ExpressionKind::Identifier(name),
                    },
                    value: Some(Expression {
                        position: Position {
                            line: 1,
                            column: 17
                        },
                        value: ExpressionKind::Identifier(someident),
                    }),
                },
            },],
        );
    }

    #[test]
    fn test_function_declaration() {
        let mut agent = Agent::new();
        let name = agent.intern_string("test");
        let ident_a = agent.intern_string("a");
        let ident_b = agent.intern_string("b");
        let ident_c = agent.intern_string("c");
        let ident_hello = agent.intern_string("hello");

        let input = "
function test(a, b, c) {
    let hello = b;
}
";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.filter_map(|s| s.ok()).collect::<Vec<_>>(),
            vec![Statement {
                position: Position { line: 2, column: 1 },
                value: StatementKind::FunctionDeclaration {
                    name: Expression {
                        position: Position {
                            line: 2,
                            column: 10
                        },
                        value: ExpressionKind::Identifier(name),
                    },
                    parameters: vec![
                        Expression {
                            position: Position {
                                line: 2,
                                column: 15
                            },
                            value: ExpressionKind::Identifier(ident_a),
                        },
                        Expression {
                            position: Position {
                                line: 2,
                                column: 18
                            },
                            value: ExpressionKind::Identifier(ident_b),
                        },
                        Expression {
                            position: Position {
                                line: 2,
                                column: 21
                            },
                            value: ExpressionKind::Identifier(ident_c),
                        },
                    ],
                    body: vec![Statement {
                        position: Position { line: 3, column: 5 },
                        value: StatementKind::LetDeclaration {
                            name: Expression {
                                position: Position { line: 3, column: 9 },
                                value: ExpressionKind::Identifier(ident_hello),
                            },
                            value: Some(Expression {
                                position: Position {
                                    line: 3,
                                    column: 17
                                },
                                value: ExpressionKind::Identifier(ident_b),
                            }),
                        },
                    },],
                },
            },],
        );
    }

    #[test]
    fn test_parameter_list_trailing_comma() {
        let mut agent = Agent::new();
        let input = "
function test(
    this,
    func,
    has,
    many,
    parameters,
) {}
";
        let ident_test = agent.intern_string("test");
        let ident_this = agent.intern_string("this");
        let ident_func = agent.intern_string("func");
        let ident_has = agent.intern_string("has");
        let ident_many = agent.intern_string("many");
        let ident_parameters = agent.intern_string("parameters");
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 2, column: 1 },
                value: StatementKind::FunctionDeclaration {
                    name: Expression {
                        position: Position {
                            line: 2,
                            column: 10
                        },
                        value: ExpressionKind::Identifier(ident_test),
                    },
                    parameters: vec![
                        Expression {
                            position: Position { line: 3, column: 5 },
                            value: ExpressionKind::Identifier(ident_this),
                        },
                        Expression {
                            position: Position { line: 4, column: 5 },
                            value: ExpressionKind::Identifier(ident_func),
                        },
                        Expression {
                            position: Position { line: 5, column: 5 },
                            value: ExpressionKind::Identifier(ident_has),
                        },
                        Expression {
                            position: Position { line: 6, column: 5 },
                            value: ExpressionKind::Identifier(ident_many),
                        },
                        Expression {
                            position: Position { line: 7, column: 5 },
                            value: ExpressionKind::Identifier(ident_parameters),
                        },
                    ],
                    body: Vec::new(),
                },
            }),],
        );
    }

    #[test]
    fn test_missing_semicolon() {
        let mut agent = Agent::new();
        let input = "let something = someident";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Err("Expected Semicolon, found end of input".to_string()),],
        );
    }

    #[test]
    fn test_keyword_as_identifier() {
        let mut agent = Agent::new();
        let input = "let while = ok;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.next().unwrap(),
            Err("Expected Identifier, got While at 1:5".to_string()),
        );
    }

    #[test]
    fn test_if_statement() {
        let mut agent = Agent::new();
        let input = "
if truee {
    let test;
}
";
        let ident_true = agent.intern_string("truee");
        let ident_test = agent.intern_string("test");
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 2, column: 1 },
                value: StatementKind::IfStatement {
                    predicate: Expression {
                        position: Position { line: 2, column: 4 },
                        value: ExpressionKind::Identifier(ident_true),
                    },
                    then_body: vec![Statement {
                        position: Position { line: 3, column: 5 },
                        value: StatementKind::LetDeclaration {
                            name: Expression {
                                position: Position { line: 3, column: 9 },
                                value: ExpressionKind::Identifier(ident_test),
                            },
                            value: None,
                        },
                    },],
                    else_body: None,
                },
            },)],
        );
    }

    #[test]
    fn test_if_else_statement() {
        let mut agent = Agent::new();
        let input = "
if truee {
    let test;
} else {
    function test() {}
}
";
        let ident_true = agent.intern_string("truee");
        let ident_test = agent.intern_string("test");
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 2, column: 1 },
                value: StatementKind::IfStatement {
                    predicate: Expression {
                        position: Position { line: 2, column: 4 },
                        value: ExpressionKind::Identifier(ident_true),
                    },
                    then_body: vec![Statement {
                        position: Position { line: 3, column: 5 },
                        value: StatementKind::LetDeclaration {
                            name: Expression {
                                position: Position { line: 3, column: 9 },
                                value: ExpressionKind::Identifier(ident_test),
                            },
                            value: None,
                        },
                    },],
                    else_body: Some(vec![Statement {
                        position: Position { line: 5, column: 5 },
                        value: StatementKind::FunctionDeclaration {
                            name: Expression {
                                position: Position {
                                    line: 5,
                                    column: 14
                                },
                                value: ExpressionKind::Identifier(ident_test),
                            },
                            parameters: Vec::new(),
                            body: Vec::new(),
                        },
                    },]),
                },
            },)],
        );
    }

    #[test]
    fn test_if_else_if_statement() {
        let mut agent = Agent::new();
        let input = "
if truee {
    let test;
} else if test {
    function test() {}
} else {}
";
        let ident_true = agent.intern_string("truee");
        let ident_test = agent.intern_string("test");
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 2, column: 1 },
                value: StatementKind::IfStatement {
                    predicate: Expression {
                        position: Position { line: 2, column: 4 },
                        value: ExpressionKind::Identifier(ident_true),
                    },
                    then_body: vec![Statement {
                        position: Position { line: 3, column: 5 },
                        value: StatementKind::LetDeclaration {
                            name: Expression {
                                position: Position { line: 3, column: 9 },
                                value: ExpressionKind::Identifier(ident_test),
                            },
                            value: None,
                        },
                    }],
                    else_body: Some(vec![Statement {
                        position: Position { line: 4, column: 8 },
                        value: StatementKind::IfStatement {
                            predicate: Expression {
                                position: Position {
                                    line: 4,
                                    column: 11
                                },
                                value: ExpressionKind::Identifier(ident_test),
                            },
                            then_body: vec![Statement {
                                position: Position { line: 5, column: 5 },
                                value: StatementKind::FunctionDeclaration {
                                    name: Expression {
                                        position: Position {
                                            line: 5,
                                            column: 14
                                        },
                                        value: ExpressionKind::Identifier(ident_test),
                                    },
                                    parameters: Vec::new(),
                                    body: Vec::new(),
                                },
                            }],
                            else_body: Some(Vec::new()),
                        },
                    }]),
                },
            },)],
        );
    }

    #[test]
    fn test_while_statement() {
        let mut agent = Agent::new();
        let input = "while 1 { if 2 {} }";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 1, column: 1 },
                value: StatementKind::WhileStatement {
                    predicate: Expression {
                        position: Position { line: 1, column: 7 },
                        value: ExpressionKind::Integer(1),
                    },
                    body: vec![Statement {
                        position: Position {
                            line: 1,
                            column: 11
                        },
                        value: StatementKind::IfStatement {
                            predicate: Expression {
                                position: Position {
                                    line: 1,
                                    column: 14
                                },
                                value: ExpressionKind::Integer(2),
                            },
                            then_body: Vec::new(),
                            else_body: None,
                        },
                    },],
                },
            }),],
        );
    }

    #[test]
    fn test_for_statement() {
        let mut agent = Agent::new();
        let input = "
for ;; {}
for let a;; {}
for ; a; {}
for let a; a; {}
for ;; a {}
for let a;; a {}
for let a; a; a {}
";
        let ident_a = agent.intern_string("a");
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![
                Ok(Statement {
                    position: Position { line: 2, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: None,
                        predicate: None,
                        increment: None,
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 3, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: Some(Box::new(Statement {
                            position: Position { line: 3, column: 5 },
                            value: StatementKind::LetDeclaration {
                                name: Expression {
                                    position: Position { line: 3, column: 9 },
                                    value: ExpressionKind::Identifier(ident_a),
                                },
                                value: None,
                            },
                        })),
                        predicate: None,
                        increment: None,
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 4, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: None,
                        predicate: Some(Expression {
                            position: Position { line: 4, column: 7 },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        increment: None,
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 5, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: Some(Box::new(Statement {
                            position: Position { line: 5, column: 5 },
                            value: StatementKind::LetDeclaration {
                                name: Expression {
                                    position: Position { line: 5, column: 9 },
                                    value: ExpressionKind::Identifier(ident_a),
                                },
                                value: None,
                            },
                        })),
                        predicate: Some(Expression {
                            position: Position {
                                line: 5,
                                column: 12
                            },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        increment: None,
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 6, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: None,
                        predicate: None,
                        increment: Some(Expression {
                            position: Position { line: 6, column: 8 },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 7, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: Some(Box::new(Statement {
                            position: Position { line: 7, column: 5 },
                            value: StatementKind::LetDeclaration {
                                name: Expression {
                                    position: Position { line: 7, column: 9 },
                                    value: ExpressionKind::Identifier(ident_a),
                                },
                                value: None,
                            },
                        })),
                        predicate: None,
                        increment: Some(Expression {
                            position: Position {
                                line: 7,
                                column: 13
                            },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        body: Vec::new(),
                    },
                }),
                Ok(Statement {
                    position: Position { line: 8, column: 1 },
                    value: StatementKind::ForStatement {
                        initializer: Some(Box::new(Statement {
                            position: Position { line: 8, column: 5 },
                            value: StatementKind::LetDeclaration {
                                name: Expression {
                                    position: Position { line: 8, column: 9 },
                                    value: ExpressionKind::Identifier(ident_a),
                                },
                                value: None,
                            },
                        })),
                        predicate: Some(Expression {
                            position: Position {
                                line: 8,
                                column: 12
                            },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        increment: Some(Expression {
                            position: Position {
                                line: 8,
                                column: 15
                            },
                            value: ExpressionKind::Identifier(ident_a),
                        }),
                        body: Vec::new(),
                    },
                }),
            ],
        );
    }

    #[test]
    fn test_continue_statement() {
        let mut agent = Agent::new();
        let input = "continue;";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 1, column: 1 },
                value: StatementKind::ContinueStatement,
            }),],
        );
    }

    #[test]
    fn test_break_statement() {
        let mut agent = Agent::new();
        let input = "break;";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 1, column: 1 },
                value: StatementKind::BreakStatement,
            }),],
        );
    }

    #[test]
    fn test_return_statement() {
        let mut agent = Agent::new();
        let input = "
return;
return 1;
";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![
                Ok(Statement {
                    position: Position { line: 2, column: 1 },
                    value: StatementKind::ReturnStatement(None),
                }),
                Ok(Statement {
                    position: Position { line: 3, column: 1 },
                    value: StatementKind::ReturnStatement(Some(Expression {
                        position: Position { line: 3, column: 8 },
                        value: ExpressionKind::Integer(1),
                    })),
                }),
            ],
        );
    }

    #[test]
    fn test_expression_statement() {
        let mut agent = Agent::new();
        let input = "123;";
        let lexer = Lexer::new(input);
        let parser = Parser::new(&mut agent, lexer);

        assert_eq!(
            parser.collect::<Vec<_>>(),
            vec![Ok(Statement {
                position: Position { line: 1, column: 1 },
                value: StatementKind::ExpressionStatement(Expression {
                    position: Position { line: 1, column: 1 },
                    value: ExpressionKind::Integer(123),
                }),
            }),],
        );
    }

    macro_rules! test_expression {
        ($input:expr, $result:expr) => {{
            let agent = Agent::new();
            test_expression!($input, $result, agent);
        }};
        ($input:expr, $result:expr, $agent:expr) => {{
            let mut agent = $agent;
            let input = $input;
            let lexer = Lexer::new(input);
            let parser = Parser::new(&mut agent, lexer);

            assert_eq!(
                parser.collect::<Vec<_>>(),
                vec![Ok(Statement {
                    position: Position { line: 1, column: 1 },
                    value: StatementKind::ExpressionStatement(Expression {
                        position: Position { line: 1, column: 1 },
                        value: $result,
                    }),
                }),],
            );
        }};
    }

    #[test]
    fn test_identifier_expression() {
        let mut agent = Agent::new();
        let ident_a = agent.intern_string("a");
        test_expression!("a;", ExpressionKind::Identifier(ident_a), agent);
    }

    #[test]
    fn test_integer_expression() {
        test_expression!("123;", ExpressionKind::Integer(123));
    }

    #[test]
    fn test_double_expression() {
        test_expression!("1.23;", ExpressionKind::Double(1.23));
    }
}
