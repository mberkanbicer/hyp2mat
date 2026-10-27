//! HyperLynx `.hyp` parser — port of `hyp2mat/lib/scan.ll` and
//! `hyp2mat/lib/parse.yy`.
//!
//! The original uses Flex/Bison. This is a hand-written lexer and recursive
//! descent parser that follows the same grammar and the same `exec_*`
//! semantics. Two scanner quirks are preserved because they are load-bearing
//! for real HyperLynx files:
//!
//! * Text after a closing `)` or `}` on the same line is a comment and is
//!   skipped (Eagle writes `(PERIMETER_SEGMENT ...) Wires: From Board` and
//!   `(? REF="TP1" ...)  R000 X=...`).
//! * Lines whose first character is `*` are comments.
//!
//! Syntax errors are reported in the native `line N: ...` style and recovery
//! continues with the next record, exactly like the Bison error productions.

use crate::format::format_cpp_float;
use crate::hypfile::{
    Attribute, Device, GeomSettings, HypFile, HypPin, HypPoint, HypPolygon, HypPolygonType,
    LayerType, Net, NetClass, Pad, PadShape, PadType, Padstack, PinFunction, Supply,
    UnroutedSegment,
};
use crate::model::PCB;

/// A parse diagnostic, formatted like the native `yyerror` output.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Source line for syntax errors; `0` for messages that the native code
    /// prints as-is (for example `warning: padstack without metal: ...`).
    pub line: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn raw(message: impl Into<String>) -> Self {
        Diagnostic {
            line: 0,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Sym(char),
    Word(String),
    Str(String),
    Num(f64),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    tok: Tok,
    line: usize,
    text: String,
}

/// Parse a complete run of non-whitespace characters as a number with an
/// optional unit suffix (native `SIMPLE_FLOAT` + `FLOAT_*` rules).
///
/// Returns `None` when the run is not a number, so the lexer can fall back to
/// a string token (flex longest-match behaviour).
fn parse_number_run(chars: &mut [char], followed_by_whitespace: bool) -> Option<f64> {
    let len = chars.len();
    let mut i = 0;
    if i < len && (chars[i] == '+' || chars[i] == '-') {
        i += 1;
    }
    let mut digits = false;
    let mut dot = false;
    while i < len {
        let c = chars[i];
        if c.is_ascii_digit() {
            digits = true;
            i += 1;
        } else if c == '.' && !dot {
            dot = true;
            i += 1;
        } else {
            break;
        }
    }
    if !digits {
        return None;
    }

    if i < len && (chars[i] == 'e' || chars[i] == 'E') {
        let saved = i;
        let mut j = i + 1;
        if j < len && (chars[j] == '+' || chars[j] == '-') {
            j += 1;
        }
        let exponent_start = j;
        while j < len && chars[j].is_ascii_digit() {
            j += 1;
        }
        if j == exponent_start {
            i = saved;
        } else {
            i = j;
        }
    }

    let number_text: String = chars[..i].iter().collect();
    let value: f64 = number_text.parse().ok()?;

    if i == len {
        return Some(value);
    }

    // A unit suffix requires trailing whitespace in the native scanner
    // (`FLOAT_SUFFIX [A-Za-z]*{WS}+`).
    if !followed_by_whitespace {
        return None;
    }
    let suffix: String = chars[i..].iter().collect();
    if !suffix.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let multiplier = match suffix.chars().next()? {
        'Y' => 1e24,
        'Z' => 1e21,
        'E' => 1e18,
        'P' => 1e15,
        'T' => 1e12,
        'G' => 1e9,
        'M' => 1e6,
        'K' | 'k' => 1e3,
        'm' => 1e-3,
        'u' | 'U' => 1e-6,
        'n' | 'N' => 1e-9,
        'p' => 1e-12,
        'f' | 'F' => 1e-15,
        'a' | 'A' => 1e-18,
        'z' => 1e-21,
        'y' => 1e-24,
        _ => return None,
    };
    Some(value * multiplier)
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    at_line_start: bool,
    skip_to_eol: bool,
}

fn is_word_break(c: char) -> bool {
    c.is_whitespace() || matches!(c, '{' | '}' | '(' | ')' | '=' | ',' | '"')
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            at_line_start: true,
            skip_to_eol: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.at_line_start = true;
        } else {
            self.at_line_start = false;
        }
        Some(c)
    }

    fn skip_line(&mut self) {
        while let Some(c) = self.peek() {
            self.bump();
            if c == '\n' {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        if self.skip_to_eol {
            self.skip_to_eol = false;
            self.skip_line();
        }
        loop {
            match self.peek() {
                None => {
                    return Token {
                        tok: Tok::Eof,
                        line: self.line,
                        text: String::new(),
                    }
                }
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('*') if self.at_line_start => {
                    self.skip_line();
                }
                _ => break,
            }
        }

        let line = self.line;
        let c = self.peek().expect("not eof");
        let mut raw_text: Option<String> = None;
        let tok = match c {
            '{' | '}' | '(' | ')' | ',' | '=' => {
                self.bump();
                if c == ')' || c == '}' {
                    self.skip_to_eol = true;
                }
                Tok::Sym(c)
            }
            '"' => {
                self.bump();
                let mut value = String::new();
                loop {
                    match self.peek() {
                        None => break,
                        Some('"') => {
                            if self.peek_at(1) == Some('"') {
                                self.bump();
                                self.bump();
                                value.push('"');
                            } else {
                                self.bump();
                                break;
                            }
                        }
                        Some(_) => {
                            let ch = self.bump().expect("char");
                            value.push(ch);
                        }
                    }
                }
                Tok::Str(value)
            }
            c if c.is_ascii_digit()
                || ((c == '+' || c == '-')
                    && self
                        .peek_at(1)
                        .is_some_and(|n| n.is_ascii_digit() || n == '.')) =>
            {
                // The scanner uses longest-match: a run such as `028-015E` is
                // a string, not a number followed by junk.
                let start = self.pos;
                let mut run = String::new();
                while let Some(ch) = self.peek() {
                    if is_word_break(ch) {
                        break;
                    }
                    run.push(self.bump().expect("char"));
                }
                let mut chars: Vec<char> = run.chars().collect();
                match parse_number_run(&mut chars, self.peek().is_some_and(char::is_whitespace)) {
                    Some(value) => {
                        raw_text = Some(run);
                        Tok::Num(value)
                    }
                    None => {
                        self.pos = start;
                        for _ in 0..run.chars().count() {
                            self.bump();
                        }
                        Tok::Word(run)
                    }
                }
            }
            _ => {
                let mut word = String::new();
                while let Some(ch) = self.peek() {
                    if is_word_break(ch) {
                        break;
                    }
                    word.push(self.bump().expect("char"));
                }
                if word.is_empty() {
                    // Unknown single character; consume it to make progress.
                    let ch = self.bump().expect("char");
                    Tok::Word(ch.to_string())
                } else {
                    Tok::Word(word)
                }
            }
        };

        let text = match &tok {
            Tok::Sym(c) => c.to_string(),
            Tok::Word(w) => w.clone(),
            Tok::Str(s) => s.clone(),
            Tok::Num(n) => raw_text.unwrap_or_else(|| format!("{n}")),
            Tok::Eof => String::new(),
        };
        Token { tok, line, text }
    }
}

/// A record parameter value.
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Num(f64),
    Text(String),
}

impl Value {
    fn as_text(&self) -> Option<String> {
        match self {
            Value::Text(t) => Some(t.clone()),
            Value::Num(_) => None,
        }
    }
}

type Params = Vec<(String, Value)>;

fn params_get_num(params: &Params, name: &str) -> Option<f64> {
    params
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| match v {
            Value::Num(n) => Some(*n),
            _ => None,
        })
}

fn params_get_text(params: &Params, name: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| v.as_text())
}

fn params_get_bool(params: &Params, name: &str) -> Option<bool> {
    params_get_text(params, name).map(|t| matches!(t.as_str(), "YES" | "yes"))
}

fn params_has(params: &Params, name: &str) -> bool {
    params.iter().any(|(k, _)| k == name)
}

/// Parser state.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    hyp: HypFile,
    diagnostics: Vec<Diagnostic>,
}

/// Parse a HyperLynx file into the intermediate model.
pub fn parse_hypfile(input: &str, geometry: GeomSettings) -> (HypFile, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let eof = token.tok == Tok::Eof;
        tokens.push(token);
        if eof {
            break;
        }
    }
    let mut parser = Parser {
        tokens,
        pos: 0,
        hyp: HypFile::new(geometry),
        diagnostics: Vec::new(),
    };
    parser.parse_file();
    (parser.hyp, parser.diagnostics)
}

/// Parse a HyperLynx file and convert it to the PCB model with default
/// options. This is the historical public entry point.
pub fn parse_hyp(input: &str) -> Result<PCB, String> {
    let (hyp, diagnostics) = parse_hypfile(input, GeomSettings::default());
    let options = crate::pcb::Options::default();
    let (pcb, _warnings) = crate::pcb::read_hyp(&hyp, &options)?;
    let _ = diagnostics;
    Ok(pcb)
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_at(&self, offset: usize) -> &Token {
        &self.tokens[(self.pos + offset).min(self.tokens.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn at_eof(&self) -> bool {
        self.peek().tok == Tok::Eof
    }

    fn check_sym(&self, c: char) -> bool {
        matches!(self.peek().tok, Tok::Sym(s) if s == c)
    }

    fn eat_sym(&mut self, c: char) -> Result<(), String> {
        if self.check_sym(c) {
            self.bump();
            Ok(())
        } else {
            Err(self.unexpected(&format!("'{c}'")))
        }
    }

    fn check_word(&self, word: &str) -> bool {
        matches!(&self.peek().tok, Tok::Word(w) if w == word)
    }

    fn peek_word(&self) -> Option<String> {
        match &self.peek().tok {
            Tok::Word(w) => Some(w.clone()),
            _ => None,
        }
    }

    fn eat_word(&mut self, word: &str) -> Result<(), String> {
        if self.check_word(word) {
            self.bump();
            Ok(())
        } else {
            Err(self.unexpected(word))
        }
    }

    fn unexpected(&self, expected: &str) -> String {
        let token = self.peek();
        let found = match &token.tok {
            Tok::Eof => "end of file".to_string(),
            Tok::Sym(c) => format!("'{c}'"),
            Tok::Word(w) => w.clone(),
            Tok::Str(s) => format!("\"{s}\""),
            Tok::Num(n) => format!("{n}"),
        };
        format!(
            "syntax error, unexpected {} at '{}' (expected {})",
            found, token.text, expected
        )
    }

    fn error_here(&mut self, expected: &str) -> String {
        let message = self.unexpected(expected);
        let line = self.peek().line;
        self.diagnostics.push(Diagnostic { line, message });
        self.unexpected(expected)
    }

    fn take_string(&mut self) -> Result<String, String> {
        match self.peek().tok.clone() {
            Tok::Str(s) => {
                self.bump();
                Ok(s)
            }
            Tok::Word(w) => {
                self.bump();
                Ok(w)
            }
            Tok::Sym('(') => {
                // scan.ll accepts an alphanumeric name in parentheses as a
                // string, e.g. `(Net0)`.
                if let (Tok::Word(w), Tok::Sym(')')) =
                    (self.peek_at(1).tok.clone(), self.peek_at(2).tok.clone())
                {
                    if w.chars().all(|c| c.is_ascii_alphanumeric()) {
                        self.bump();
                        self.bump();
                        self.bump();
                        return Ok(format!("({w})"));
                    }
                }
                Err(self.unexpected("string"))
            }
            _ => Err(self.unexpected("string")),
        }
    }

    fn take_number(&mut self) -> Result<f64, String> {
        match self.peek().tok.clone() {
            Tok::Num(n) => {
                self.bump();
                Ok(n)
            }
            _ => Err(self.unexpected("number")),
        }
    }

    /// Read `KEY = VALUE` pairs until a stop symbol.
    fn read_params(&mut self, stop: char) -> Result<Params, String> {
        let mut params = Vec::new();
        loop {
            if self.check_sym(stop) || self.at_eof() {
                return Ok(params);
            }
            if self.check_sym(',') {
                self.bump();
                continue;
            }
            if self.check_sym('(') {
                // A record that starts a nested construct (polygon lines).
                return Ok(params);
            }
            let key = match self.peek_word() {
                Some(word) => word,
                None => return Err(self.unexpected("parameter name")),
            };
            self.bump();
            if !self.check_sym('=') {
                return Err(self.unexpected("'='"));
            }
            self.bump();
            let value = match self.peek().tok.clone() {
                Tok::Num(n) => {
                    self.bump();
                    Value::Num(n)
                }
                Tok::Str(s) => {
                    self.bump();
                    Value::Text(s)
                }
                Tok::Word(w) => {
                    self.bump();
                    Value::Text(w)
                }
                _ => return Err(self.unexpected("parameter value")),
            };
            params.push((key, value));
        }
    }

    /// Device parameters: the native scanner switches to a string state after
    /// `VAL=`, so even numeric values are kept verbatim (`VAL=100n` becomes the
    /// string `"100n"`, not `1e-7`).
    fn read_device_params(&mut self) -> Result<Params, String> {
        let mut params = Vec::new();
        loop {
            if self.check_sym(')') || self.at_eof() {
                return Ok(params);
            }
            if self.check_sym(',') {
                self.bump();
                continue;
            }
            let key = match self.peek_word() {
                Some(word) => word,
                None => return Err(self.unexpected("parameter name")),
            };
            self.bump();
            if !self.check_sym('=') {
                return Err(self.unexpected("'='"));
            }
            self.bump();
            if key == "VAL" {
                let text = self.peek().text.clone();
                match self.peek().tok {
                    Tok::Num(_) | Tok::Word(_) | Tok::Str(_) => {
                        self.bump();
                        params.push((key, Value::Text(text)));
                    }
                    _ => return Err(self.unexpected("parameter value")),
                }
            } else {
                let value = match self.peek().tok.clone() {
                    Tok::Num(n) => {
                        self.bump();
                        Value::Num(n)
                    }
                    Tok::Str(s) => {
                        self.bump();
                        Value::Text(s)
                    }
                    Tok::Word(w) => {
                        self.bump();
                        Value::Text(w)
                    }
                    _ => return Err(self.unexpected("parameter value")),
                };
                params.push((key, value));
            }
        }
    }

    fn skip_balanced(&mut self, open: char, close: char) {
        // The opening symbol has already been consumed.
        let mut depth = 1;
        while !self.at_eof() {
            if self.check_sym(open) {
                depth += 1;
            } else if self.check_sym(close) {
                depth -= 1;
                if depth == 0 {
                    self.bump();
                    return;
                }
            }
            self.bump();
        }
    }

    fn skip_record(&mut self) {
        // Called with an unconsumed '(' or '{'.
        if self.check_sym('(') {
            self.bump();
            self.skip_balanced('(', ')');
        } else if self.check_sym('{') {
            self.bump();
            self.skip_balanced('{', '}');
        } else {
            self.bump();
        }
    }

    fn parse_file(&mut self) {
        while !self.at_eof() {
            if !self.check_sym('{') {
                self.error_here("'{'");
                self.bump();
                continue;
            }
            let start = self.pos;
            self.bump();
            let word = match self.peek_word() {
                Some(w) => w,
                None => {
                    self.error_here("section name");
                    self.pos = start;
                    self.skip_record();
                    continue;
                }
            };
            let result = match word.as_str() {
                "VERSION" => self.parse_version(),
                "DATA_MODE" => self.parse_data_mode(),
                "UNITS" => self.parse_units(),
                "PLANE_SEP" => self.parse_plane_sep(),
                "BOARD_FILE" => self.parse_board_file(),
                "BOARD" => self.parse_board(),
                "STACKUP" => self.parse_stackup(),
                "DEVICES" => self.parse_devices(),
                "SUPPLIES" => self.parse_supplies(),
                "PADSTACK" => self.parse_padstack(),
                "NET" => self.parse_net(),
                "NET_CLASS" => self.parse_net_class(),
                "END" => self.parse_end(),
                "KEY" => self.parse_key(),
                other => Err(format!("syntax error, unexpected {other} at '{other}'")),
            };
            if let Err(message) = result {
                let line = self.tokens[start].line;
                self.diagnostics.push(Diagnostic { line, message });
                self.pos = start;
                self.skip_record();
            }
        }
    }

    fn parse_version(&mut self) -> Result<(), String> {
        self.eat_word("VERSION")?;
        self.eat_sym('=')?;
        let version = self.take_number()?;
        self.eat_sym('}')?;
        self.hyp.version = version;
        if version < 2.0 {
            self.diagnostics
                .push(Diagnostic::raw("warning: version 1.x deprecated"));
        }
        Ok(())
    }

    fn parse_data_mode(&mut self) -> Result<(), String> {
        self.eat_word("DATA_MODE")?;
        self.eat_sym('=')?;
        let mode = self.take_string()?;
        self.hyp.detailed = match mode.as_str() {
            "SIMPLIFIED" => false,
            "DETAILED" => true,
            other => return Err(format!("expected mode, found '{other}'")),
        };
        self.skip_to_rbrace()?;
        Ok(())
    }

    fn parse_units(&mut self) -> Result<(), String> {
        self.eat_word("UNITS")?;
        self.eat_sym('=')?;
        let system = self.take_string()?;
        let english = match system.as_str() {
            "ENGLISH" => true,
            "METRIC" => false,
            other => return Err(format!("expected unit system, found '{other}'")),
        };
        let metal = self.take_string()?;
        let metal_weight = match metal.as_str() {
            "WEIGHT" => true,
            "LENGTH" => false,
            other => return Err(format!("expected weight/length, found '{other}'")),
        };
        self.skip_to_rbrace()?;
        self.hyp.set_units(english, metal_weight);
        Ok(())
    }

    fn skip_to_rbrace(&mut self) -> Result<(), String> {
        while !self.check_sym('}') {
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            self.bump();
        }
        self.bump();
        Ok(())
    }

    fn parse_plane_sep(&mut self) -> Result<(), String> {
        self.eat_word("PLANE_SEP")?;
        self.eat_sym('=')?;
        let separation = self.take_number()?;
        self.eat_sym('}')?;
        self.hyp.board.plane_separation = separation * self.hyp.unit;
        Ok(())
    }

    fn parse_board_file(&mut self) -> Result<(), String> {
        self.eat_word("BOARD_FILE")?;
        // `{BOARD_FILE}` in the grammar; tolerate `{BOARD_FILE=name}`.
        if self.check_sym('=') {
            self.bump();
            self.take_string()?;
        }
        self.eat_sym('}')?;
        Ok(())
    }

    fn parse_board(&mut self) -> Result<(), String> {
        self.eat_word("BOARD")?;
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            let result = match self.peek_word().as_deref() {
                Some("PERIMETER_SEGMENT") => self.parse_perimeter_segment(),
                Some("PERIMETER_ARC") => self.parse_perimeter_arc(),
                Some("A") => self.parse_board_attribute(),
                Some(other) => Err(format!("unexpected board record '{other}'")),
                None => Err(self.unexpected("board record")),
            };
            if let Err(message) = result {
                self.diagnostics.push(Diagnostic {
                    line: self.peek().line,
                    message,
                });
                self.skip_balanced('(', ')');
            }
        }
    }

    fn parse_perimeter_segment(&mut self) -> Result<(), String> {
        self.eat_word("PERIMETER_SEGMENT")?;
        let params = self.read_params(')')?;
        let (x1, y1, x2, y2) = self.require_line(&params)?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let segment = HypPolygon {
            vertex: vec![
                HypPoint::new(x1 * unit, y1 * unit),
                HypPoint::new(x2 * unit, y2 * unit),
            ],
            polygon_type: HypPolygonType::Copper,
            id: -1,
            positive: true,
            width: 0.0,
            ..HypPolygon::default()
        };
        self.hyp.add_perimeter_polygon(segment);
        Ok(())
    }

    fn parse_perimeter_arc(&mut self) -> Result<(), String> {
        self.eat_word("PERIMETER_ARC")?;
        let params = self.read_params(')')?;
        let (x1, y1, x2, y2, xc, yc, r) = self.require_arc(&params)?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let arc = self.hyp.arc2poly(
            x1 * unit,
            y1 * unit,
            x2 * unit,
            y2 * unit,
            xc * unit,
            yc * unit,
            r * unit,
            false,
        );
        self.hyp.add_perimeter_polygon(arc);
        Ok(())
    }

    fn parse_board_attribute(&mut self) -> Result<(), String> {
        self.eat_word("A")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let name = params_get_text(&params, "N").ok_or("board attribute needs N=")?;
        let value = params_get_text(&params, "V").ok_or("board attribute needs V=")?;
        self.hyp.board.attribute.push(Attribute { name, value });
        Ok(())
    }

    fn require_line(&self, params: &Params) -> Result<(f64, f64, f64, f64), String> {
        Ok((
            params_get_num(params, "X1").ok_or("missing X1")?,
            params_get_num(params, "Y1").ok_or("missing Y1")?,
            params_get_num(params, "X2").ok_or("missing X2")?,
            params_get_num(params, "Y2").ok_or("missing Y2")?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn require_arc(&self, params: &Params) -> Result<(f64, f64, f64, f64, f64, f64, f64), String> {
        let (x1, y1, x2, y2) = self.require_line(params)?;
        Ok((
            x1,
            y1,
            x2,
            y2,
            params_get_num(params, "XC").ok_or("missing XC")?,
            params_get_num(params, "YC").ok_or("missing YC")?,
            params_get_num(params, "R").ok_or("missing R")?,
        ))
    }

    fn parse_stackup(&mut self) -> Result<(), String> {
        self.eat_word("STACKUP")?;
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            let result = match self.peek_word().as_deref() {
                Some("OPTIONS") => self.parse_options(),
                Some("SIGNAL") => self.parse_layer(LayerType::Signal),
                Some("DIELECTRIC") => self.parse_layer(LayerType::Dielectric),
                Some("PLANE") => self.parse_layer(LayerType::Plane),
                Some(other) => Err(format!("unexpected stackup record '{other}'")),
                None => Err(self.unexpected("stackup record")),
            };
            if let Err(message) = result {
                self.diagnostics.push(Diagnostic {
                    line: self.peek().line,
                    message,
                });
                self.skip_balanced('(', ')');
            }
        }
    }

    fn parse_options(&mut self) -> Result<(), String> {
        self.eat_word("OPTIONS")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        if params_has(&params, "USE_DIE_FOR_METAL") {
            self.hyp.use_die_for_metal =
                params_get_bool(&params, "USE_DIE_FOR_METAL").unwrap_or(false);
        }
        Ok(())
    }

    fn parse_layer(&mut self, layer_type: LayerType) -> Result<(), String> {
        match layer_type {
            LayerType::Signal => {
                self.eat_word("SIGNAL")?;
            }
            LayerType::Dielectric => {
                self.eat_word("DIELECTRIC")?;
            }
            LayerType::Plane => {
                self.eat_word("PLANE")?;
            }
        }
        let params = self.read_params(')')?;
        self.eat_sym(')')?;

        let thickness = params_get_num(&params, "T").ok_or("thickness missing")?;
        let layer_name = params_get_text(&params, "L");
        let material_name = params_get_text(&params, "M");

        match layer_type {
            LayerType::Dielectric => {
                // In a DIELECTRIC record `C` is the dielectric constant.
                let epsilon_r =
                    params_get_num(&params, "C").or_else(|| params_get_num(&params, "ER"));
                self.hyp.add_dielectric_layer(
                    thickness,
                    epsilon_r,
                    params_get_num(&params, "LT"),
                    params_get_bool(&params, "CO?"),
                    params_get_bool(&params, "PR?"),
                    layer_name,
                    material_name,
                );
            }
            _ => {
                // Native `add_metal_layer` rejects dielectric-only properties.
                if params_has(&params, "CO?") {
                    self.diagnostics
                        .push(Diagnostic::raw("only dielectric layers conformal"));
                    return Ok(());
                }
                if params_has(&params, "PR?") {
                    self.diagnostics
                        .push(Diagnostic::raw("only dielectric layers prepreg"));
                    return Ok(());
                }
                if layer_name.is_none() {
                    self.diagnostics
                        .push(Diagnostic::raw("missing signal layer name"));
                }
                self.hyp.add_metal_layer(
                    layer_type,
                    thickness,
                    params_get_num(&params, "P"),
                    params_get_num(&params, "C").or_else(|| params_get_num(&params, "BR")),
                    params_get_num(&params, "TC"),
                    params_get_num(&params, "ER"),
                    params_get_num(&params, "LT"),
                    layer_name,
                    material_name,
                    params_get_num(&params, "PS"),
                );
            }
        }
        Ok(())
    }

    fn parse_devices(&mut self) -> Result<(), String> {
        self.eat_word("DEVICES")?;
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            if let Err(message) = self.parse_device() {
                self.diagnostics.push(Diagnostic {
                    line: self.peek().line,
                    message,
                });
                self.skip_balanced('(', ')');
            }
        }
    }

    fn parse_device(&mut self) -> Result<(), String> {
        let device_type = self.take_string()?;
        self.eat_word("REF")?;
        self.eat_sym('=')?;
        let ref_ = self.take_string()?;
        let params = self.read_device_params()?;
        self.eat_sym(')')?;
        let name = params_get_text(&params, "NAME").unwrap_or_default();
        let layer_name = params_get_text(&params, "L").unwrap_or_default();
        let package = params_get_text(&params, "PKG").unwrap_or_default();
        // `VAL=` always yields a string in the native scanner (`STATE_STRING`).
        let (value_float, value_float_set, value_string, value_string_set) =
            match params.iter().find(|(k, _)| k == "VAL") {
                Some((_, Value::Text(s))) => (0.0, false, s.clone(), true),
                Some((_, Value::Num(n))) => (0.0, false, format_cpp_float(*n), true),
                None => (0.0, false, String::new(), false),
            };
        if layer_name.is_empty() {
            return Err("device layer not set".to_string());
        }
        self.hyp.device.push(Device {
            device_type,
            ref_,
            name,
            value_float,
            value_float_set,
            value_string,
            value_string_set,
            layer_name,
            package,
        });
        Ok(())
    }

    fn parse_supplies(&mut self) -> Result<(), String> {
        self.eat_word("SUPPLIES")?;
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            let result = (|| -> Result<(), String> {
                self.eat_word("S")?;
                let params = self.read_params(')')?;
                self.eat_sym(')')?;
                let name = params_get_text(&params, "NAME").ok_or("supply needs NAME=")?;
                let value_float = params_get_num(&params, "VAL").ok_or("supply needs VAL=")?;
                let voltage_specified = params_get_bool(&params, "V?").unwrap_or(false);
                let conversion = params_get_bool(&params, "C?").unwrap_or(false);
                self.hyp.supply.push(Supply {
                    name,
                    value_float,
                    voltage_specified,
                    conversion,
                });
                Ok(())
            })();
            if let Err(message) = result {
                self.diagnostics.push(Diagnostic {
                    line: self.peek().line,
                    message,
                });
                self.skip_balanced('(', ')');
            }
        }
    }

    fn parse_padstack(&mut self) -> Result<(), String> {
        self.eat_word("PADSTACK")?;
        self.eat_sym('=')?;
        let padstack_name = self.take_string()?;
        let mut drill_size = 0.0;
        if self.check_sym(',') {
            self.bump();
            if matches!(self.peek().tok, Tok::Num(_)) {
                drill_size = self.take_number()?;
            }
        }
        let mut created = false;
        loop {
            if self.check_sym('}') {
                self.bump();
                break;
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            let start_diag = self.diagnostics.len();
            let start = self.pos;
            self.bump();
            let result = self.parse_padstack_element(&padstack_name, drill_size, &mut created);
            if let Err(message) = result {
                self.diagnostics.truncate(start_diag);
                self.diagnostics.push(Diagnostic {
                    line: self.tokens[start].line,
                    message,
                });
                self.pos = start;
                self.skip_record();
            }
        }
        if created {
            let warnings = self.hyp.finish_padstack();
            for warning in warnings {
                self.diagnostics.push(Diagnostic::raw(warning));
            }
        }
        Ok(())
    }

    fn parse_padstack_element(
        &mut self,
        padstack_name: &str,
        drill_size: f64,
        created: &mut bool,
    ) -> Result<(), String> {
        let layer_name = self.take_string()?;
        self.eat_sym(',')?;
        // Optional pad shape: Altium sometimes prints an empty field.
        let pad_shape = if self.check_sym(',') {
            -1.0
        } else {
            let value = self.take_number()?;
            self.eat_sym(',')?;
            value
        };
        let pad_sx = self.take_number()?;
        self.eat_sym(',')?;
        let pad_sy = self.take_number()?;
        self.eat_sym(',')?;
        let pad_angle = self.take_number()?;

        let mut pad_type: Option<PadType> = None;
        let mut thermal_clear_shape = PadShape::Oval;
        let mut thermal_clear_sx = 0.0;
        let mut thermal_clear_sy = 0.0;
        let mut thermal_clear_angle = 0.0;
        if self.check_sym(',') {
            self.bump();
            match self.peek_word().as_deref() {
                Some("M") => {
                    self.bump();
                    pad_type = Some(PadType::Metal);
                }
                Some("A") => {
                    self.bump();
                    pad_type = Some(PadType::Antipad);
                }
                _ => {
                    thermal_clear_shape = Self::pad_shape_from_number(self.take_number()?)?;
                    self.eat_sym(',')?;
                    thermal_clear_sx = self.take_number()?;
                    self.eat_sym(',')?;
                    thermal_clear_sy = self.take_number()?;
                    self.eat_sym(',')?;
                    thermal_clear_angle = self.take_number()?;
                    self.eat_sym(',')?;
                    self.eat_word("T")?;
                    pad_type = Some(PadType::ThermalRelief);
                }
            }
        }
        self.eat_sym(')')?;

        if !*created {
            self.hyp.padstack.push(Padstack {
                padstack_name: padstack_name.to_string(),
                drill_size: drill_size * self.hyp.unit,
                pads: Vec::new(),
            });
            *created = true;
        }

        let pad_shape = Self::pad_shape_from_number(pad_shape)?;
        // `ADEF` implies an antipad when no explicit pad type is given.
        let pad_type = pad_type.unwrap_or(if layer_name == "ADEF" {
            PadType::Antipad
        } else {
            PadType::Metal
        });

        let unit = self.hyp.unit;
        let pad = Pad {
            layer_name,
            pad_type,
            pad_shape,
            pad_sx: pad_sx * unit,
            pad_sy: pad_sy * unit,
            pad_angle,
            thermal_clear_shape,
            thermal_clear_sx: thermal_clear_sx * unit,
            thermal_clear_sy: thermal_clear_sy * unit,
            thermal_clear_angle,
        };
        if let Some(padstack) = self.hyp.padstack.last_mut() {
            padstack.pads.push(pad);
        }
        Ok(())
    }

    fn pad_shape_from_number(value: f64) -> Result<PadShape, String> {
        match value {
            0.0 | -1.0 => Ok(PadShape::Oval),
            1.0 => Ok(PadShape::Rectangular),
            2.0 => Ok(PadShape::Oblong),
            other => Err(format!("unknown pad shape {other}")),
        }
    }

    fn parse_net(&mut self) -> Result<(), String> {
        self.eat_word("NET")?;
        self.eat_sym('=')?;
        let net_name = self.take_string()?;
        self.hyp.net.push(Net {
            net_name,
            metal: Default::default(),
            plane_separation: -1.0,
            via: Vec::new(),
            unrouted_segment: Vec::new(),
            pin: Vec::new(),
            attribute: Vec::new(),
        });
        if self.check_word("PS") {
            self.bump();
            self.eat_sym('=')?;
            let separation = self.take_number()? * self.hyp.unit;
            if let Some(net) = self.hyp.net.last_mut() {
                net.plane_separation = separation;
            }
        }
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if self.check_sym('(') {
                let start = self.pos;
                self.bump();
                let result = match self.peek_word().as_deref() {
                    Some("SEG") => self.parse_seg(),
                    Some("ARC") => self.parse_arc(),
                    Some("VIA") => self.parse_via(),
                    Some("PIN") => self.parse_pin(),
                    Some("PAD") => self.parse_pad(),
                    Some("USEG") => self.parse_useg(),
                    Some("A") => self.parse_net_attribute(),
                    Some(other) => Err(format!("unexpected net record '{other}'")),
                    None => Err(self.unexpected("net record")),
                };
                if let Err(message) = result {
                    self.diagnostics.push(Diagnostic {
                        line: self.tokens[start].line,
                        message,
                    });
                    self.pos = start;
                    self.skip_record();
                }
            } else if self.check_sym('{') {
                let start = self.pos;
                self.bump();
                let result = match self.peek_word().as_deref() {
                    Some("POLYGON") => self.parse_polygon(),
                    Some("POLYVOID") => self.parse_polyvoid(),
                    Some("POLYLINE") => self.parse_polyline(),
                    Some(other) => Err(format!("unexpected polygon record '{other}'")),
                    None => Err(self.unexpected("polygon record")),
                };
                if let Err(message) = result {
                    self.diagnostics.push(Diagnostic {
                        line: self.tokens[start].line,
                        message,
                    });
                    self.pos = start;
                    self.skip_record();
                }
            } else {
                return Err(self.unexpected("net record"));
            }
        }
    }

    fn parse_seg(&mut self) -> Result<(), String> {
        self.eat_word("SEG")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let (x1, y1, x2, y2) = self.require_line(&params)?;
        let width = params_get_num(&params, "W").unwrap_or(0.0);
        let layer_name = params_get_text(&params, "L").unwrap_or_default();
        let plane_separation = params_get_num(&params, "PS").unwrap_or(-1.0);
        let left_plane_separation = params_get_num(&params, "LPS").unwrap_or(-1.0);
        let unit = self.hyp.unit;
        let mut polygon =
            self.hyp
                .segment2poly(x1 * unit, y1 * unit, x2 * unit, y2 * unit, width * unit);
        polygon.polygon_type = HypPolygonType::Copper;
        polygon.layer_name = layer_name;
        polygon.width = 0.0;
        polygon.positive = true;
        polygon.plane_separation = if plane_separation < 0.0 {
            -1.0
        } else {
            plane_separation * unit
        };
        polygon.left_plane_separation = if left_plane_separation < 0.0 {
            -1.0
        } else {
            left_plane_separation * unit
        };
        self.hyp.add_polygon(polygon)
    }

    fn parse_arc(&mut self) -> Result<(), String> {
        self.eat_word("ARC")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let (x1, y1, x2, y2, xc, yc, r) = self.require_arc(&params)?;
        let width = params_get_num(&params, "W").unwrap_or(0.0);
        let layer_name = params_get_text(&params, "L").unwrap_or_default();
        let plane_separation = params_get_num(&params, "PS").unwrap_or(-1.0);
        let left_plane_separation = params_get_num(&params, "LPS").unwrap_or(-1.0);
        let unit = self.hyp.unit;
        let arc = self.hyp.arc2poly(
            x1 * unit,
            y1 * unit,
            x2 * unit,
            y2 * unit,
            xc * unit,
            yc * unit,
            r * unit,
            true,
        );
        for pair in arc.vertex.windows(2) {
            let mut polygon =
                self.hyp
                    .segment2poly(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width * unit);
            polygon.polygon_type = HypPolygonType::Copper;
            polygon.layer_name = layer_name.clone();
            polygon.width = 0.0;
            polygon.positive = true;
            polygon.plane_separation = if plane_separation < 0.0 {
                -1.0
            } else {
                plane_separation * unit
            };
            polygon.left_plane_separation = if left_plane_separation < 0.0 {
                -1.0
            } else {
                left_plane_separation * unit
            };
            self.hyp.add_polygon(polygon)?;
        }
        Ok(())
    }

    fn parse_via(&mut self) -> Result<(), String> {
        self.eat_word("VIA")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let x = params_get_num(&params, "X").ok_or("via needs X=")? * unit;
        let y = params_get_num(&params, "Y").ok_or("via needs Y=")? * unit;
        match params_get_text(&params, "P") {
            Some(padstack_name) => {
                self.exec_via_padstack(x, y, &padstack_name, &params);
                Ok(())
            }
            None => {
                self.exec_via_v1(x, y, &params);
                Ok(())
            }
        }
    }

    fn exec_via_padstack(&mut self, x: f64, y: f64, padstack_name: &str, params: &Params) {
        let padstack = self
            .hyp
            .padstack
            .iter()
            .find(|p| p.padstack_name == padstack_name)
            .cloned();
        let Some(padstack) = padstack else {
            return;
        };
        let mut layer1 = params_get_text(params, "L1");
        let mut layer2 = params_get_text(params, "L2");
        if layer1.is_none() {
            layer1 = padstack
                .pads
                .first()
                .map(|p| p.layer_name.clone())
                .or_else(|| self.hyp.stackup.first().map(|l| l.layer_name.clone()));
        }
        if layer2.is_none() {
            layer2 = padstack
                .pads
                .last()
                .map(|p| p.layer_name.clone())
                .or_else(|| self.hyp.stackup.last().map(|l| l.layer_name.clone()));
        }
        let layer1 = layer1.unwrap_or_default();
        let layer2 = layer2.unwrap_or_default();
        self.hyp
            .add_via(x, y, &layer2, &layer1, padstack.drill_size / 2.0);
        for pad in &padstack.pads {
            let _ = self.hyp.add_pad(x, y, pad);
        }
    }

    fn exec_via_v1(&mut self, x: f64, y: f64, params: &Params) {
        let unit = self.hyp.unit;
        let drill_size = params_get_num(params, "D").unwrap_or(0.0) * unit;
        let mut layer1 = params_get_text(params, "L1");
        let mut layer2 = params_get_text(params, "L2");
        if layer1.is_none() {
            layer1 = self.hyp.stackup.first().map(|l| l.layer_name.clone());
        }
        if layer2.is_none() {
            layer2 = self.hyp.stackup.last().map(|l| l.layer_name.clone());
        }
        let layer1 = layer1.unwrap_or_default();
        let layer2 = layer2.unwrap_or_default();
        self.hyp.add_via(x, y, &layer2, &layer1, drill_size / 2.0);

        let shapes = [params_get_text(params, "S1"), params_get_text(params, "S2")];
        if shapes[0].is_none()
            || params_get_num(params, "S1X").is_none()
            || params_get_num(params, "S1Y").is_none()
            || params_get_num(params, "A1").is_none()
            || shapes[1].is_none()
            || params_get_num(params, "S2X").is_none()
            || params_get_num(params, "S2Y").is_none()
            || params_get_num(params, "A2").is_none()
        {
            return;
        }
        for (index, layer_name) in [layer1, layer2].iter().enumerate() {
            let suffix = index + 1;
            let shape = shapes[index].clone().unwrap_or_default();
            let pad = Pad {
                layer_name: layer_name.clone(),
                pad_type: PadType::Metal,
                pad_shape: match shape.as_str() {
                    "RECT" => PadShape::Rectangular,
                    "OBLONG" => PadShape::Oblong,
                    _ => PadShape::Oval,
                },
                pad_sx: params_get_num(params, &format!("S{suffix}X")).unwrap_or(0.0) * unit,
                pad_sy: params_get_num(params, &format!("S{suffix}Y")).unwrap_or(0.0) * unit,
                pad_angle: params_get_num(params, &format!("A{suffix}")).unwrap_or(0.0),
                thermal_clear_shape: PadShape::Oval,
                thermal_clear_sx: 0.0,
                thermal_clear_sy: 0.0,
                thermal_clear_angle: 0.0,
            };
            let _ = self.hyp.add_pad(x, y, &pad);
        }
    }

    fn parse_pin(&mut self) -> Result<(), String> {
        self.eat_word("PIN")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let x = params_get_num(&params, "X").ok_or("pin needs X=")? * unit;
        let y = params_get_num(&params, "Y").ok_or("pin needs Y=")? * unit;
        let pin_reference = params_get_text(&params, "R").ok_or("pin needs R=")?;
        let padstack_name = params_get_text(&params, "P").unwrap_or_default();
        let pin_function = match params_get_text(&params, "F").as_deref() {
            Some("SIM_IN") => PinFunction::SimIn,
            Some("SIM_OUT") => PinFunction::SimOut,
            _ => PinFunction::SimBoth,
        };
        let pin = HypPin {
            x,
            y,
            pin_reference,
            padstack_name: padstack_name.clone(),
            pin_function,
        };
        if let Some(net) = self.hyp.net.last_mut() {
            net.pin.push(pin);
        }
        if !padstack_name.is_empty() {
            let padstack = self
                .hyp
                .padstack
                .iter()
                .find(|p| p.padstack_name == padstack_name)
                .cloned();
            if let Some(padstack) = padstack {
                let bottom = self
                    .hyp
                    .stackup
                    .last()
                    .map(|l| l.layer_name.clone())
                    .unwrap_or_default();
                let top = self
                    .hyp
                    .stackup
                    .first()
                    .map(|l| l.layer_name.clone())
                    .unwrap_or_default();
                self.hyp
                    .add_via(x, y, &bottom, &top, padstack.drill_size / 2.0);
                for pad in &padstack.pads {
                    let _ = self.hyp.add_pad(x, y, pad);
                }
            }
        }
        Ok(())
    }

    fn parse_pad(&mut self) -> Result<(), String> {
        self.eat_word("PAD")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let x = params_get_num(&params, "X").ok_or("pad needs X=")? * unit;
        let y = params_get_num(&params, "Y").ok_or("pad needs Y=")? * unit;
        let layer_name = params_get_text(&params, "L").unwrap_or_default();
        // Native quirk: `exec_pad` reads the `via_pad1_*` fields, which the
        // grammar of a `PAD` record never sets (`S`/`SX`/`SY`/`A` fill
        // `via_pad_*` instead). The result is always a zero-sized oval pad,
        // which the polygon library subsequently drops. Preserved verbatim.
        let pad = Pad {
            layer_name,
            pad_type: PadType::Metal,
            pad_shape: PadShape::Oval,
            pad_sx: 0.0,
            pad_sy: 0.0,
            pad_angle: 0.0,
            thermal_clear_shape: PadShape::Oval,
            thermal_clear_sx: 0.0,
            thermal_clear_sy: 0.0,
            thermal_clear_angle: 0.0,
        };
        self.hyp.add_pad(x, y, &pad)
    }

    fn parse_useg(&mut self) -> Result<(), String> {
        self.eat_word("USEG")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let unit = self.hyp.unit;
        let segment = UnroutedSegment {
            x1: params_get_num(&params, "X1").ok_or("useg needs X1=")? * unit,
            y1: params_get_num(&params, "Y1").ok_or("useg needs Y1=")? * unit,
            layer1_name: params_get_text(&params, "L1").unwrap_or_default(),
            x2: params_get_num(&params, "X2").ok_or("useg needs X2=")? * unit,
            y2: params_get_num(&params, "Y2").ok_or("useg needs Y2=")? * unit,
            layer2_name: params_get_text(&params, "L2").unwrap_or_default(),
            zlayer_name: params_get_text(&params, "ZL").unwrap_or_default(),
            zlayer_name_set: params_has(&params, "ZL"),
            width: params_get_num(&params, "ZW").unwrap_or(0.0) * unit,
            impedance: params_get_num(&params, "Z").unwrap_or(0.0),
            impedance_set: params_has(&params, "Z"),
            delay: params_get_num(&params, "D").unwrap_or(0.0),
            resistance: params_get_num(&params, "R").unwrap_or(0.0),
        };
        if let Some(net) = self.hyp.net.last_mut() {
            net.unrouted_segment.push(segment);
        }
        Ok(())
    }

    fn parse_net_attribute(&mut self) -> Result<(), String> {
        self.eat_word("A")?;
        let params = self.read_params(')')?;
        self.eat_sym(')')?;
        let name = params_get_text(&params, "N").ok_or("attribute needs N=")?;
        let value = params_get_text(&params, "V").ok_or("attribute needs V=")?;
        if let Some(net) = self.hyp.net.last_mut() {
            net.attribute.push(Attribute { name, value });
        }
        Ok(())
    }

    fn parse_polygon(&mut self) -> Result<(), String> {
        self.eat_word("POLYGON")?;
        let params = self.read_params_polygon()?;
        self.parse_polygon_body(&params)
    }

    /// Polygon records read parameters until the first nested `(` (a `LINE` or
    /// `CURVE` subrecord) or the closing `}`.
    fn read_params_polygon(&mut self) -> Result<Params, String> {
        let mut params = Vec::new();
        loop {
            if self.check_sym('}') || self.at_eof() {
                return Ok(params);
            }
            if self.check_sym('(') {
                return Ok(params);
            }
            if self.check_sym(',') {
                self.bump();
                continue;
            }
            let key = match self.peek_word() {
                Some(word) => word,
                None => return Err(self.unexpected("parameter name")),
            };
            // A bare word without '=' ends the parameter list.
            if !matches!(self.peek_at(1).tok, Tok::Sym('=')) {
                return Ok(params);
            }
            self.bump();
            self.bump();
            let value = match self.peek().tok.clone() {
                Tok::Num(n) => {
                    self.bump();
                    Value::Num(n)
                }
                Tok::Str(s) => {
                    self.bump();
                    Value::Text(s)
                }
                Tok::Word(w) => {
                    self.bump();
                    Value::Text(w)
                }
                _ => return Err(self.unexpected("parameter value")),
            };
            params.push((key, value));
        }
    }

    fn parse_polygon_body(&mut self, params: &Params) -> Result<(), String> {
        let unit = self.hyp.unit;
        let layer_name = params_get_text(params, "L");
        if layer_name.is_none() {
            return Err("expected polygon layer L =".to_string());
        }
        let width = params_get_num(params, "W").unwrap_or(0.0);
        let polygon_type = match params_get_text(params, "T").as_deref() {
            Some("POUR") => HypPolygonType::Pour,
            Some("PLANE") => HypPolygonType::Plane,
            Some("COPPER") => HypPolygonType::Copper,
            _ => HypPolygonType::Plane,
        };
        let id = params_get_num(params, "ID")
            .map(|n| n as i32)
            .ok_or("expected polygon id ID =")?;
        let x = params_get_num(params, "X").ok_or("polygon needs X=")?;
        let y = params_get_num(params, "Y").ok_or("polygon needs Y=")?;

        let mut polygon = HypPolygon {
            vertex: vec![HypPoint::new(x * unit, y * unit)],
            polygon_type,
            id,
            positive: true,
            width: width * unit,
            layer_name: layer_name.unwrap_or_default(),
            ..HypPolygon::default()
        };
        self.parse_lines_and_curves(&mut polygon)?;
        self.hyp.add_polygon(polygon)
    }

    fn parse_polyvoid(&mut self) -> Result<(), String> {
        self.eat_word("POLYVOID")?;
        let params = self.read_params_polygon()?;
        let unit = self.hyp.unit;
        let id = params_get_num(&params, "ID")
            .map(|n| n as i32)
            .ok_or("expected polygon id ID =")?;
        let x = params_get_num(&params, "X").ok_or("polyvoid needs X=")?;
        let y = params_get_num(&params, "Y").ok_or("polyvoid needs Y=")?;

        // Inherit layer, width and type from the parent polygon.
        let parent = self
            .hyp
            .net
            .last()
            .and_then(|net| net.metal.get(&id))
            .and_then(|edges| edges.iter().find(|e| e.positive))
            .cloned();
        let Some(parent) = parent else {
            return Err(
                "polyvoid does not have parent polygon or polyline with same id".to_string(),
            );
        };
        let mut polygon = HypPolygon {
            vertex: vec![HypPoint::new(x * unit, y * unit)],
            polygon_type: parent.polygon_type,
            id,
            positive: false,
            width: parent.width,
            layer_name: parent.layer_name,
            plane_separation: parent.plane_separation,
            left_plane_separation: parent.left_plane_separation,
            net_name: parent.net_name,
        };
        self.parse_lines_and_curves(&mut polygon)?;
        self.hyp.add_polygon(polygon)
    }

    fn parse_polyline(&mut self) -> Result<(), String> {
        self.eat_word("POLYLINE")?;
        let params = self.read_params_polygon()?;
        let unit = self.hyp.unit;
        let layer_name = params_get_text(&params, "L").ok_or("expected polygon layer L =")?;
        let width = params_get_num(&params, "W").ok_or("expected polygon width W =")?;
        let polygon_type = match params_get_text(&params, "T").as_deref() {
            Some("POUR") => HypPolygonType::Pour,
            Some("PLANE") => HypPolygonType::Plane,
            Some("COPPER") => HypPolygonType::Copper,
            _ => HypPolygonType::Plane,
        };
        let id = params_get_num(&params, "ID")
            .map(|n| n as i32)
            .ok_or("expected polygon id ID =")?;
        let x = params_get_num(&params, "X").ok_or("polyline needs X=")?;
        let y = params_get_num(&params, "Y").ok_or("polyline needs Y=")?;
        let mut polygon = HypPolygon {
            vertex: vec![HypPoint::new(x * unit, y * unit)],
            polygon_type,
            id,
            positive: true,
            width: 0.0,
            layer_name,
            ..HypPolygon::default()
        };
        self.parse_lines_and_curves(&mut polygon)?;
        // Draw the polyline as a sequence of trace segments.
        for pair in polygon.vertex.windows(2) {
            let mut segment =
                self.hyp
                    .segment2poly(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width * unit);
            segment.id = polygon.id;
            segment.polygon_type = polygon.polygon_type;
            segment.positive = true;
            segment.layer_name = polygon.layer_name.clone();
            segment.width = 0.0;
            self.hyp.add_polygon(segment)?;
        }
        Ok(())
    }

    fn parse_lines_and_curves(&mut self, polygon: &mut HypPolygon) -> Result<(), String> {
        let unit = self.hyp.unit;
        loop {
            if self.check_sym('}') {
                self.bump();
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            let word = self.peek_word().ok_or("expected line or curve")?;
            match word.as_str() {
                "LINE" => {
                    self.bump();
                    let params = self.read_params(')')?;
                    self.eat_sym(')')?;
                    let x = params_get_num(&params, "X").ok_or("line needs X=")?;
                    let y = params_get_num(&params, "Y").ok_or("line needs Y=")?;
                    polygon.vertex.push(HypPoint::new(x * unit, y * unit));
                }
                "ARC" | "CURVE" => {
                    self.bump();
                    let params = self.read_params(')')?;
                    self.eat_sym(')')?;
                    let (x1, y1, x2, y2, xc, yc, r) = self.require_arc(&params)?;
                    let arc = self.hyp.arc2poly(
                        x1 * unit,
                        y1 * unit,
                        x2 * unit,
                        y2 * unit,
                        xc * unit,
                        yc * unit,
                        r * unit,
                        false,
                    );
                    let last = *polygon.vertex.last().ok_or("empty polygon")?;
                    let dist1 =
                        (last.x - arc.vertex[0].x).powi(2) + (last.y - arc.vertex[0].y).powi(2);
                    let last_arc = *arc.vertex.last().expect("arc has vertices");
                    let dist2 = (last.x - last_arc.x).powi(2) + (last.y - last_arc.y).powi(2);
                    if dist1 <= dist2 {
                        polygon.vertex.extend_from_slice(&arc.vertex);
                    } else {
                        polygon.vertex.extend(arc.vertex.iter().rev().copied());
                    }
                }
                other => return Err(format!("expected line or curve, found '{other}'")),
            }
        }
    }

    fn parse_net_class(&mut self) -> Result<(), String> {
        self.eat_word("NET_CLASS")?;
        self.eat_sym('=')?;
        let net_class_name = self.take_string()?;
        let mut net_class = NetClass {
            net_class_name,
            ..NetClass::default()
        };
        loop {
            if self.check_sym('}') {
                self.bump();
                self.hyp.net_class.push(net_class);
                return Ok(());
            }
            if self.at_eof() {
                return Err(self.unexpected("'}'"));
            }
            if !self.check_sym('(') {
                return Err(self.unexpected("'('"));
            }
            self.bump();
            let word = self.peek_word().ok_or("expected net class record")?;
            match word.as_str() {
                "N" => {
                    self.bump();
                    self.eat_word("N")?;
                    self.eat_sym('=')?;
                    net_class.net_class_element.push(self.take_string()?);
                    self.eat_sym(')')?;
                }
                "A" => {
                    self.bump();
                    let params = self.read_params(')')?;
                    self.eat_sym(')')?;
                    let name = params_get_text(&params, "N").ok_or("attribute needs N=")?;
                    let value = params_get_text(&params, "V").ok_or("attribute needs V=")?;
                    net_class.attribute.push(Attribute { name, value });
                }
                other => return Err(format!("unexpected net class record '{other}'")),
            }
        }
    }

    fn parse_end(&mut self) -> Result<(), String> {
        self.eat_word("END")?;
        self.eat_sym('}')?;
        self.hyp.ended = true;
        Ok(())
    }

    fn parse_key(&mut self) -> Result<(), String> {
        self.eat_word("KEY")?;
        self.eat_sym('=')?;
        let key = self.take_string()?;
        self.eat_sym('}')?;
        self.hyp.key = key;
        Ok(())
    }
}

/// Parser tests.
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> (HypFile, Vec<Diagnostic>) {
        parse_hypfile(input, GeomSettings::default())
    }

    #[test]
    fn empty_input_is_an_empty_model() {
        let (hyp, diagnostics) = parse("");
        assert!(hyp.stackup.is_empty());
        assert!(hyp.net.is_empty());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn version_and_units() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}  Compatible with Hyperlynx version 2.10.\n{UNITS=ENGLISH LENGTH}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.version, 2.10);
        assert_eq!(hyp.unit, 0.0254);
        assert_eq!(hyp.metal_thickness_unit, 0.0254);
    }

    #[test]
    fn data_mode_and_plane_sep() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{DATA_MODE=DETAILED}\n{PLANE_SEP=0.02}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(hyp.detailed);
        assert!((hyp.board.plane_separation - 0.0002).abs() < 1e-12);
    }

    #[test]
    fn stackup_layers_are_positioned() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n(DIELECTRIC T=0.100 C=4.5 L=DL)\n(SIGNAL T=0.00035 L=Bottom)\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.stackup.len(), 3);
        assert_eq!(hyp.stackup[0].layer_name, "Top");
        assert!(hyp.stackup[0].z1 > 0.0);
        assert_eq!(hyp.stackup[2].z0, 0.0);
    }

    #[test]
    fn board_perimeter_and_cutout() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{BOARD\n(PERIMETER_SEGMENT X1=0 Y1=0 X2=1 Y2=0) Wires\n(PERIMETER_SEGMENT X1=1 Y1=0 X2=1 Y2=1)\n(PERIMETER_SEGMENT X1=1 Y1=1 X2=0 Y2=1)\n(PERIMETER_SEGMENT X1=0 Y1=1 X2=0 Y2=0)\n(A N=\"name\" V=\"value\")\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.board.edge.len(), 1);
        assert_eq!(hyp.board.edge[0].vertex.len(), 8);
        assert_eq!(hyp.board.attribute.len(), 1);
    }

    #[test]
    fn device_and_supply() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{DEVICES\n(R REF=\"R1\" NAME=\"0603\" VAL=50 L=\"Top\")\n}\n{SUPPLIES\n(S NAME=VCC VAL=3.3 V?=YES C?=NO)\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.device.len(), 1);
        assert_eq!(hyp.device[0].ref_, "R1");
        assert_eq!(hyp.device[0].value_string, "50");
        assert!(hyp.device[0].value_string_set);
        assert_eq!(hyp.supply.len(), 1);
        assert!(hyp.supply[0].voltage_specified);
        assert!((hyp.supply[0].value_float - 3.3).abs() < 1e-12);
    }

    #[test]
    fn padstack_expands_default_pads() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n(DIELECTRIC T=0.100 L=DL)\n(SIGNAL T=0.00035 L=Bottom)\n}\n{PADSTACK=THRU\n,0.04\n(MDEF,0,0.08,0.08,0.0)\n(Bottom,1,0.08,0.08,0.0)\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.padstack.len(), 1);
        assert_eq!(hyp.padstack[0].pads.len(), 2);
        assert!(hyp.padstack[0].pads.iter().all(|p| p.layer_name != "MDEF"));
    }

    #[test]
    fn net_with_segment_and_polygon() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n}\n{NET=N1\n(SEG X1=0 Y1=0 X2=1 Y2=0 W=0.02 L=Top)\n{POLYGON L=\"Top\" T=COPPER W=0 ID=1 X=0.1 Y=0.1\n(LINE X=0.1 Y=0.2)\n(LINE X=0.2 Y=0.2)\n(LINE X=0.2 Y=0.1)\n}\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.net.len(), 1);
        let net = &hyp.net[0];
        assert_eq!(net.metal.len(), 2);
        assert!(net.metal.values().any(|edges| edges.len() == 1));
        let polygon_edges = net
            .metal
            .iter()
            .find(|(id, _)| **id == 1)
            .map(|(_, edges)| edges.clone())
            .expect("polygon id 1");
        assert_eq!(polygon_edges[0].vertex.len(), 4);
        assert_eq!(polygon_edges[0].polygon_type, HypPolygonType::Copper);
    }

    #[test]
    fn polyvoid_inherits_parent() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n}\n{NET=N1\n{POLYGON L=\"Top\" T=POUR W=0 ID=1 X=0 Y=0\n(LINE X=0 Y=1)\n(LINE X=1 Y=1)\n(LINE X=1 Y=0)\n}\n{POLYVOID ID=1 X=0.2 Y=0.2\n(LINE X=0.2 Y=0.4)\n(LINE X=0.4 Y=0.4)\n(LINE X=0.4 Y=0.2)\n}\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let edges = &hyp.net[0].metal[&1];
        assert_eq!(edges.len(), 2);
        assert!(edges[0].positive);
        assert!(!edges[1].positive);
        assert_eq!(edges[1].layer_name, "Top");
    }

    #[test]
    fn via_padstack_creates_pads_and_hole() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n(DIELECTRIC T=0.100 L=DL)\n(SIGNAL T=0.00035 L=Bottom)\n}\n{PADSTACK=V\n,0.04\n(Top,1,0.08,0.08,0.0)\n(Bottom,1,0.08,0.08,0.0)\n}\n{NET=N1\n(VIA X=0.5 Y=0.5 P=V)\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.net[0].via.len(), 1);
        let via = &hyp.net[0].via[0];
        assert_eq!(via.radius, 0.0002);
        assert_eq!(via.z0, 0.0);
        // Pads are added to the net's metal map with generated negative ids.
        assert_eq!(hyp.net[0].metal.len(), 2);
    }

    #[test]
    fn arcs_and_rotated_pads() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n(DIELECTRIC T=0.100 L=DL)\n(SIGNAL T=0.00035 L=Bottom)\n}\n{PADSTACK=P\n(Top,1,0.2,0.1,45.0)\n}\n{NET=N1\n(PIN X=0.5 Y=0.5 R=U1.1 P=P)\n(ARC X1=0.1 Y1=0.1 X2=0.2 Y2=0.2 XC=0.1 YC=0.2 R=0.1 W=0.02 L=Top)\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.net[0].pin.len(), 1);
        // One pad polygon plus six arc segments (native draws a 270 degree
        // clockwise arc for these end points).
        assert_eq!(hyp.net[0].metal.len(), 7);
    }

    #[test]
    fn net_class_and_key() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{NET_CLASS=CLK\n(N N=CLK_P)\n(A N=\"z\" V=\"50\")\n}\n{END}\n{KEY=abc}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.net_class.len(), 1);
        assert_eq!(hyp.net_class[0].net_class_element, vec!["CLK_P"]);
        assert_eq!(hyp.key, "abc");
    }

    #[test]
    fn syntax_error_is_reported_and_recovery_continues() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{BOGUS 1}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n}\n",
        );
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("syntax error"));
        assert_eq!(hyp.stackup.len(), 1);
    }

    #[test]
    fn scanner_skips_trailing_text_after_records() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{DEVICES\n(? REF=\"TP1\" NAME=\"TPTP10SQ\" L=\"Top\")  R000 X=0.1 Y=0.2 : Lib: x\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.device.len(), 1);
        assert_eq!(hyp.device[0].name, "TPTP10SQ");
    }

    #[test]
    fn quoted_strings_unescape_double_quotes() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{DEVICES\n(? REF=\"U\"\"1\" NAME=\"a b\" L=\"Top\")\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.device[0].ref_, "U\"1");
    }

    #[test]
    fn device_values_are_always_strings() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{DEVICES\n(C REF=\"C1\" VAL=100n L=\"Top\")\n(R REF=\"R1\" VAL=50 L=\"Top\")\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.device[0].value_string, "100n");
        assert!(hyp.device[0].value_string_set);
        assert!(!hyp.device[0].value_float_set);
        assert_eq!(hyp.device[1].value_string, "50");
    }

    #[test]
    fn number_suffixes_follow_the_native_lexer() {
        // `T=35u` is a floating point number with a micro suffix.
        let (hyp, diagnostics) =
            parse("{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=35u L=Top)\n}\n");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!((hyp.stackup[0].thickness - 35e-6 * 0.01).abs() < 1e-18);
    }

    #[test]
    fn metal_layer_rejects_dielectric_only_options() {
        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(SIGNAL T=0.00035 CO?=YES L=Top)\n}\n",
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "only dielectric layers conformal");
        assert!(hyp.stackup.is_empty());

        let (hyp, diagnostics) = parse(
            "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(PLANE T=0.00035 PR?=YES L=GND)\n}\n",
        );
        assert_eq!(diagnostics[0].message, "only dielectric layers prepreg");
        assert!(hyp.stackup.is_empty());
    }

    #[test]
    fn metal_layer_without_name_warns_and_continues() {
        let (hyp, diagnostics) =
            parse("{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{STACKUP\n(PLANE T=0.00035)\n}\n");
        assert_eq!(diagnostics[0].message, "missing signal layer name");
        assert_eq!(hyp.stackup.len(), 1);
        assert_eq!(hyp.stackup[0].layer_name, "");
    }

    #[test]
    fn key_with_hex_like_digits_is_a_string() {
        // Flex longest-match: `028-015E-4E5D` is a string, not a number.
        let (hyp, diagnostics) = parse("{VERSION=2.10}\n{KEY=028-015E-4E5D}\n");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(hyp.key, "028-015E-4E5D");
    }
}
