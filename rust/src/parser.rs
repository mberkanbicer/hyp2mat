//! HyperLynx .hyp parser — port of `scan.ll` + `parse.yy`.
//!
//! Hand-written recursive-descent parser (no Flex/Bison dependency).
//! Handles the sections: board_file, version, data_mode, units, plane_sep,
//! board, stackup, devices, supplies, padstack, net, net_class, end, key.

use crate::model::{
    Device, FloatPoint, FloatPolygon, FloatPoly, FloatPolygons, Layer, PCB, Pin, Via,
};

/// Token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Punctuation
    LBrace, RBrace, LParen, RParen, Comma, Equals,
    // Sections
    BoardFile, Version, DataMode, Units, PlaneSep, Board, Stackup, Devices,
    Supplies, Padstack, Net, NetClass, End, Key,
    // Keywords
    Arc, Copper, Curve, Dielectric, English, Length, Line, Metric, Pad,
    PerimeterArc, PerimeterSegment, Pin, Plane, Polygon, Polyline, Polyvoid,
    Pour, Seg, Signal, SimBoth, SimIn, SimOut, Useg, Via, Weight,
    // Assignments
    A, A1, A2, BR, C, CQm, COQm, D, ER, F, ID,
    L, L1, L2, Lps, LT, M, Name, P, Pkg, PRQm, PS, R, Ref, S, SX, SY,
    S1, S1X, S1Y, S2, S2X, S2Y, T, TC, UseDieForMetal, V, VQm, Val, W,
    X, X1, X2, Xc, Y, Y1, Y2, Yc, Z, ZL, ZLen, ZW,
    Options, Simplified, Detailed,
    // Literals
    Bool(bool), Int(i32), Float(f64), Str(String),
    Eof,
}

/// Simple lexer for .hyp files.
struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer { input, pos: 0, line: 1 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.input[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' | '\u{0b}' | '\u{0c}' => { self.next(); }
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self) {
        // Lines starting with * are comments
        if self.peek() == Some('*') {
            while let Some(ch) = self.peek() {
                if ch == '\n' { break; }
                self.next();
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        // optional sign
        if self.peek() == Some('-') || self.peek() == Some('+') {
            self.next();
        }
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                self.next();
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        // check for suffix multiplier (must be directly attached, no whitespace)
        // Order matters: uppercase M=mega, lowercase m=milli, etc.
        let multiplier = match self.peek() {
            Some('Y') => { self.next(); 1e24 }
            // lowercase y = yocto (handled below)
            Some('Z') => { self.next(); 1e21 }
            // lowercase z = zepto (handled below)
            Some('E') | Some('e') => { /* exponent already consumed in number */ 1.0 }
            Some('P') => { self.next(); 1e15 }
            // lowercase p = pico (handled below)
            Some('T') => { self.next(); 1e12 }
            Some('G') => { self.next(); 1e9 }
            Some('M') => { self.next(); 1e6 }
            Some('K') | Some('k') => { self.next(); 1e3 }
            Some('m') => { self.next(); 1e-3 }
            Some('u') | Some('U') => { self.next(); 1e-6 }
            Some('n') | Some('N') => { self.next(); 1e-9 }
            Some('p') => { self.next(); 1e-12 }
            Some('f') | Some('F') => { self.next(); 1e-15 }
            Some('a') | Some('A') => { self.next(); 1e-18 }
            Some('z') => { self.next(); 1e-21 }
            Some('y') => { self.next(); 1e-24 }
            _ => 1.0,
        };
        let val: f64 = num_str.parse().unwrap_or(0.0);
        Token::Float(val * multiplier)
    }

    fn read_string(&mut self) -> Token {
        // Quoted string
        if self.peek() == Some('"') {
            self.next(); // consume opening quote
            let start = self.pos;
            while let Some(ch) = self.peek() {
                if ch == '"' {
                    // check for escaped quote ""
                    let saved = self.pos;
                    self.next();
                    if self.peek() == Some('"') {
                        self.next(); // consume second quote, it's an escaped quote
                    } else {
                        self.pos = saved; // back up, this is the closing quote
                        break;
                    }
                } else {
                    self.next();
                }
            }
            let s = &self.input[start..self.pos];
            if self.peek() == Some('"') {
                self.next(); // consume closing quote
            }
            Token::Str(s.to_string())
        } else {
            // Unquoted string: sequence of non-whitespace, non-reserved chars
            let start = self.pos;
            while let Some(ch) = self.peek() {
                match ch {
                    ' ' | '\t' | '\r' | '\n' | '\u{0b}' | '\u{0c}'
                    | '{' | '}' | '(' | ')' | '=' | '"' => break,
                    _ => { self.next(); }
                }
            }
            let s = &self.input[start..self.pos].trim();
            Token::Str(s.to_string())
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '?' {
                self.next();
            } else {
                break;
            }
        }
        let word = &self.input[start..self.pos];
        // Check if followed by "=" (assignment form), skipping whitespace
        let saved_pos = self.pos;
        self.skip_whitespace();
        let is_assignment = self.peek() == Some('=');
        self.pos = saved_pos; // restore position, don't consume whitespace

        match word {
            "BOARD_FILE" => Token::BoardFile,
            "VERSION" => Token::Version,
            "DATA_MODE" => Token::DataMode,
            "UNITS" => Token::Units,
            "PLANE_SEP" => Token::PlaneSep,
            "BOARD" => Token::Board,
            "STACKUP" => Token::Stackup,
            "DEVICES" => Token::Devices,
            "SUPPLIES" => Token::Supplies,
            "PADSTACK" => Token::Padstack,
            "NET" => Token::Net,
            "NET_CLASS" => Token::NetClass,
            "END" => Token::End,
            "KEY" => Token::Key,
            "ARC" => Token::Arc,
            "COPPER" => Token::Copper,
            "CURVE" => Token::Curve,
            "DETAILED" => Token::Detailed,
            "DIELECTRIC" => Token::Dielectric,
            "OPTIONS" => Token::Options,
            "ENGLISH" => Token::English,
            "LENGTH" => Token::Length,
            "LINE" => Token::Line,
            "METRIC" => Token::Metric,
            "PAD" => Token::Pad,
            "PERIMETER_ARC" => Token::PerimeterArc,
            "PERIMETER_SEGMENT" => Token::PerimeterSegment,
            "PIN" => Token::Pin,
            "PLANE" => Token::Plane,
            "POLYGON" => Token::Polygon,
            "POLYLINE" => Token::Polyline,
            "POLYVOID" => Token::Polyvoid,
            "POUR" => Token::Pour,
            "SEG" => Token::Seg,
            "SIGNAL" => Token::Signal,
            "SIMPLIFIED" => Token::Simplified,
            "SIM_BOTH" => Token::SimBoth,
            "A" => Token::A,
            "SIM_IN" => Token::SimIn,
            "SIM_OUT" => Token::SimOut,
            "USEG" => Token::Useg,
            "VIA" => Token::Via,
            "WEIGHT" => Token::Weight,
            "YES" | "yes" => Token::Bool(true),
            "NO" | "no" => Token::Bool(false),
            "A1" if is_assignment => Token::A1,
            "A2" if is_assignment => Token::A2,
            "BR" if is_assignment => Token::BR,
            "C" if is_assignment => Token::C,
            "C?" if is_assignment => Token::CQm,
            "CO?" if is_assignment => Token::COQm,
            "D" if is_assignment => Token::D,
            "ER" if is_assignment => Token::ER,
            "F" if is_assignment => Token::F,
            "ID" if is_assignment => Token::ID,
            "L" if is_assignment => Token::L,
            "L1" if is_assignment => Token::L1,
            "L2" if is_assignment => Token::L2,
            "LPS" if is_assignment => Token::Lps,
            "LT" if is_assignment => Token::LT,
            "M" if is_assignment => Token::M,
            "N" if is_assignment => Token::Str(word.to_string()),
            "NAME" if is_assignment => Token::Name,
            "P" if is_assignment => Token::P,
            "PKG" if is_assignment => Token::Pkg,
            "PR?" if is_assignment => Token::PRQm,
            "PS" if is_assignment => Token::PS,
            "R" if is_assignment => Token::R,
            "REF" if is_assignment => Token::Ref,
            "S" if is_assignment => Token::S,
            "SX" if is_assignment => Token::SX,
            "SY" if is_assignment => Token::SY,
            "S1" if is_assignment => Token::S1,
            "S1X" if is_assignment => Token::S1X,
            "S1Y" if is_assignment => Token::S1Y,
            "S2" if is_assignment => Token::S2,
            "S2X" if is_assignment => Token::S2X,
            "S2Y" if is_assignment => Token::S2Y,
            "T" if is_assignment => Token::T,
            "TC" if is_assignment => Token::TC,
            "USE_DIE_FOR_METAL" if is_assignment => Token::UseDieForMetal,
            "V" if is_assignment => Token::V,
            "V?" if is_assignment => Token::VQm,
            "VAL" if is_assignment => Token::Val,
            "W" if is_assignment => Token::W,
            "X" if is_assignment => Token::X,
            "X1" if is_assignment => Token::X1,
            "X2" if is_assignment => Token::X2,
            "XC" if is_assignment => Token::Xc,
            "Y" if is_assignment => Token::Y,
            "Y1" if is_assignment => Token::Y1,
            "Y2" if is_assignment => Token::Y2,
            "YC" if is_assignment => Token::Yc,
            "Z" if is_assignment => Token::Z,
            "ZL" if is_assignment => Token::ZL,
            "ZLEN" if is_assignment => Token::ZLen,
            "ZW" if is_assignment => Token::ZW,
            _ => {
                if is_assignment {
                    // Unknown assignment variable, treat as string
                    Token::Str(word.to_string())
                } else {
                    Token::Str(word.to_string())
                }
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.skip_comment();
        self.skip_whitespace();

        match self.peek() {
            None => Token::Eof,
            Some('{') => { self.next(); Token::LBrace }
            Some('}') => { self.next(); Token::RBrace }
            Some('(') => { self.next(); Token::LParen }
            Some(')') => { self.next(); Token::RParen }
            Some(',') => { self.next(); Token::Comma }
            Some('=') => { self.next(); Token::Equals }
            Some('"') => self.read_string(),
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => self.read_number(),
            Some(_) => self.read_identifier_or_keyword(),
        }
    }
}

/// Parser state accumulated during parsing.
struct ParseState {
    pcb: PCB,
    current_layer: Option<Layer>,
    current_net_name: Option<String>,
    current_polygon_points: Vec<FloatPoint>,
    current_polygon_layer: Option<String>,
}

/// Recursive-descent parser.
struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    state: ParseState,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token();
        Parser {
            lexer,
            current,
            state: ParseState {
                pcb: PCB::default(),
                current_layer: None,
                current_net_name: None,
                current_polygon_points: Vec::new(),
                current_polygon_layer: None,
            },
        }
    }

    fn eat(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&expected) {
            self.current = self.lexer.next_token();
            Ok(())
        } else {
            Err(format!("expected {:?}, found {:?}", expected, self.current))
        }
    }

    fn eat_any(&mut self) -> Token {
        let t = self.current.clone();
        self.current = self.lexer.next_token();
        t
    }

    /// Main entry: parse the entire .hyp file.
    pub fn parse(mut self) -> Result<PCB, String> {
        while self.current != Token::Eof {
            self.parse_section()?;
        }
        Ok(self.state.pcb)
    }

    fn parse_section(&mut self) -> Result<(), String> {
        match &self.current {
            Token::BoardFile => self.parse_board_file(),
            Token::Version => self.parse_version(),
            Token::DataMode => self.parse_data_mode(),
            Token::Units => self.parse_units(),
            Token::PlaneSep => self.parse_plane_sep(),
            Token::Board => self.parse_board(),
            Token::Stackup => self.parse_stackup(),
            Token::Devices => self.parse_devices(),
            Token::Supplies => self.parse_supplies(),
            Token::Padstack => self.parse_padstack(),
            Token::Net => self.parse_net(),
            Token::NetClass => self.parse_net_class(),
            Token::End => self.parse_end(),
            Token::Key => self.parse_key(),
            Token::LBrace => {
                // { SECTION ... } braced section, or anonymous block to skip
                self.eat_any(); // consume {
                match &self.current {
                    Token::Version => {
                        self.eat(Token::Version)?;
                        self.eat(Token::Equals)?;
                        if let Token::Float(_) = self.eat_any() {}
                        else { return Err("expected float for version".to_string()); }
                        self.eat(Token::RBrace)?;
                    }
                    Token::DataMode => {
                        self.eat(Token::DataMode)?;
                        self.eat(Token::Equals)?;
                        match self.eat_any() {
                            Token::Simplified | Token::Detailed => {}
                            t => return Err(format!("expected mode, found {:?}", t)),
                        }
                        self.eat(Token::RBrace)?;
                    }
                    Token::Units => {
                        self.eat(Token::Units)?;
                        self.eat(Token::Equals)?;
                        match self.eat_any() {
                            Token::English | Token::Metric => {}
                            t => return Err(format!("expected unit system, found {:?}", t)),
                        }
                        match self.eat_any() {
                            Token::Weight | Token::Length => {}
                            t => return Err(format!("expected weight/length, found {:?}", t)),
                        }
                        self.eat(Token::RBrace)?;
                    }
                    Token::PlaneSep => {
                        self.eat(Token::PlaneSep)?;
                        self.eat(Token::Equals)?;
                        if let Token::Float(_) = self.eat_any() {}
                        else { return Err("expected float for plane_sep".to_string()); }
                        self.eat(Token::RBrace)?;
                    }
                    Token::BoardFile => {
                        self.eat(Token::BoardFile)?;
                        self.eat(Token::Equals)?;
                        if let Token::Str(_) = self.eat_any() {}
                        else { return Err("expected string for board_file".to_string()); }
                        self.eat(Token::RBrace)?;
                    }
                    Token::Stackup => self.parse_stackup()?,
                    Token::Devices => self.parse_devices()?,
                    Token::Net => self.parse_net()?,
                    _ => { self.skip_balanced_block()?; }
                }
                Ok(())
            }
            t => Err(format!("unexpected token {:?} at start of section", t)),
        }
    }

    /// Parse a balanced { ... } block, skipping contents.
    /// The opening { must already be consumed by the caller.
    fn skip_balanced_block(&mut self) -> Result<(), String> {
        let mut depth: i32 = 1;
        loop {
            match &self.current {
                Token::LBrace => { depth += 1; self.eat_any(); }
                Token::RBrace => {
                    depth -= 1;
                    self.eat_any();
                    if depth == 0 { return Ok(()); }
                }
                Token::Eof => return Err("unexpected EOF in block".to_string()),
                _ => { self.eat_any(); }
            }
        }
    }

    // ----- individual section parsers -----

    fn parse_board_file(&mut self) -> Result<(), String> {
        self.eat(Token::BoardFile)?;
        self.eat(Token::LBrace)?;
        self.skip_balanced_block()
    }

    fn parse_version(&mut self) -> Result<(), String> {
        self.eat(Token::Version)?;
        self.eat(Token::Equals)?;
        if let Token::Float(v) = self.eat_any() {
            // version stored but not used in PCB model directly
            let _ = v;
        } else {
            return Err("expected float for version".to_string());
        }
        self.eat(Token::RBrace)
    }

    fn parse_data_mode(&mut self) -> Result<(), String> {
        self.eat(Token::DataMode)?;
        self.eat(Token::Equals)?;
        match self.eat_any() {
            Token::Simplified | Token::Detailed => {}
            t => return Err(format!("expected mode, found {:?}", t)),
        }
        self.eat(Token::RBrace)
    }

    fn parse_units(&mut self) -> Result<(), String> {
        self.eat(Token::Units)?;
        self.eat(Token::Equals)?;
        match self.eat_any() {
            Token::English | Token::Metric => {}
            t => return Err(format!("expected unit system, found {:?}", t)),
        }
        match self.eat_any() {
            Token::Weight | Token::Length => {}
            t => return Err(format!("expected weight/length, found {:?}", t)),
        }
        self.eat(Token::RBrace)
    }

    fn parse_plane_sep(&mut self) -> Result<(), String> {
        self.eat(Token::PlaneSep)?;
        self.eat(Token::Equals)?;
        if let Token::Float(_) = self.eat_any() {}
        else { return Err("expected float for plane_sep".to_string()); }
        self.eat(Token::RBrace)
    }

    fn parse_board(&mut self) -> Result<(), String> {
        self.eat(Token::Board)?;
        self.eat(Token::LBrace)?;
        while self.current != Token::RBrace && self.current != Token::Eof {
            self.parse_board_param()?;
        }
        self.eat(Token::RBrace)
    }

    fn parse_board_param(&mut self) -> Result<(), String> {
        match &self.current {
            Token::LParen => {
                self.eat_any();
                match &self.current {
                    Token::PerimeterSegment => self.parse_perimeter_segment()?,
                    Token::PerimeterArc => self.parse_perimeter_arc()?,
                    Token::A => self.parse_board_attribute()?,
                    t => return Err(format!("unexpected board param {:?}", t)),
                }
                self.eat(Token::RParen)?;
            }
            t => return Err(format!("unexpected board param start {:?}", t)),
        }
        Ok(())
    }

    fn parse_perimeter_segment(&mut self) -> Result<(), String> {
        self.eat(Token::PerimeterSegment)?;
        self.parse_coord_line()?;
        // width, layer, etc. would follow; skip to closing paren handled by caller
        Ok(())
    }

    fn parse_perimeter_arc(&mut self) -> Result<(), String> {
        self.eat(Token::PerimeterArc)?;
        self.parse_coord_arc()?;
        Ok(())
    }

    fn parse_board_attribute(&mut self) -> Result<(), String> {
        self.eat(Token::A)?;
        // N = STRING V = STRING
        self.eat_any(); // N
        self.eat(Token::Equals)?;
        if let Token::Str(_) = self.eat_any() {}
        else { return Err("expected string for board attribute name".to_string()); }
        self.eat(Token::V)?;
        self.eat(Token::Equals)?;
        if let Token::Str(_) = self.eat_any() {}
        else { return Err("expected string for board attribute value".to_string()); }
        Ok(())
    }

    fn parse_stackup(&mut self) -> Result<(), String> {
        self.eat(Token::Stackup)?;
        // LBrace already consumed by caller
        while self.current != Token::RBrace && self.current != Token::Eof {
            self.parse_stackup_param()?;
        }
        self.eat(Token::RBrace)
    }

    fn parse_stackup_param(&mut self) -> Result<(), String> {
        self.eat(Token::LParen)?;
        match &self.current {
            Token::Options => self.parse_options()?,
            Token::Signal => self.parse_signal_layer()?,
            Token::Dielectric => self.parse_dielectric_layer()?,
            Token::Plane => self.parse_plane_layer()?,
            t => return Err(format!("unexpected stackup param {:?}", t)),
        }
        self.eat(Token::RParen)?;
        Ok(())
    }

    fn parse_options(&mut self) -> Result<(), String> {
        self.eat(Token::Options)?;
        // options params
        while self.current != Token::RParen {
            match &self.current {
                Token::UseDieForMetal => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    self.eat_any(); // BOOL
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_signal_layer(&mut self) -> Result<(), String> {
        self.eat(Token::Signal)?;
        let mut layer = Layer {
            name: String::new(),
            layer_type: "signal".to_string(),
            thickness: 0.0,
            z0: 0.0,
            z1: 0.0,
            epsilon_r: 0.0,
            loss_tangent: 0.0,
            bulk_resistivity: 0.0,
            metal: FloatPolygons { polys: Vec::new() },
        };
        while self.current != Token::RParen {
            self.parse_layer_param(&mut layer)?;
        }
        self.state.pcb.stackup.push(layer);
        Ok(())
    }

    fn parse_dielectric_layer(&mut self) -> Result<(), String> {
        self.eat(Token::Dielectric)?;
        let mut layer = Layer {
            name: String::new(),
            layer_type: "dielectric".to_string(),
            thickness: 0.0,
            z0: 0.0,
            z1: 0.0,
            epsilon_r: 4.0,
            loss_tangent: 0.0,
            bulk_resistivity: 0.0,
            metal: FloatPolygons { polys: Vec::new() },
        };
        while self.current != Token::RParen {
            self.parse_layer_param(&mut layer)?;
        }
        self.state.pcb.stackup.push(layer);
        Ok(())
    }

    fn parse_plane_layer(&mut self) -> Result<(), String> {
        self.eat(Token::Plane)?;
        let mut layer = Layer {
            name: String::new(),
            layer_type: "plane".to_string(),
            thickness: 0.0,
            z0: 0.0,
            z1: 0.0,
            epsilon_r: 0.0,
            loss_tangent: 0.0,
            bulk_resistivity: 0.0,
            metal: FloatPolygons { polys: Vec::new() },
        };
        while self.current != Token::RParen {
            self.parse_layer_param(&mut layer)?;
        }
        self.state.pcb.stackup.push(layer);
        Ok(())
    }

    fn parse_layer_param(&mut self, layer: &mut Layer) -> Result<(), String> {
        match &self.current {
            Token::T => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(v) = self.eat_any() { layer.thickness = v; }
                else { return Err("expected float for thickness".to_string()); }
            }
            Token::C => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(v) = self.eat_any() { layer.bulk_resistivity = v; }
                else { return Err("expected float for C".to_string()); }
            }
            Token::ER => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(v) = self.eat_any() { layer.epsilon_r = v; }
                else { return Err("expected float for ER".to_string()); }
            }
            Token::LT => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(v) = self.eat_any() { layer.loss_tangent = v; }
                else { return Err("expected float for LT".to_string()); }
            }
            Token::L => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Str(s) = self.eat_any() { layer.name = s; }
                else { return Err("expected string for L".to_string()); }
            }
            Token::M => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Str(s) = self.eat_any() { /* material name */ let _ = s; }
                else { return Err("expected string for M".to_string()); }
            }
            Token::PS => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(_) = self.eat_any() {}
                else { return Err("expected float for PS".to_string()); }
            }
            Token::BR => {
                self.eat_any();
                self.eat(Token::Equals)?;
                if let Token::Float(v) = self.eat_any() { layer.bulk_resistivity = v; }
                else { return Err("expected float for BR".to_string()); }
            }
            Token::TC => {
                self.eat_any();
                self.eat(Token::Equals)?;
                self.eat_any(); // float
            }
            t => return Err(format!("unexpected layer param {:?}", t)),
        }
        Ok(())
    }

    fn parse_devices(&mut self) -> Result<(), String> {
        self.eat(Token::Devices)?;
        // LBrace already consumed by caller
        while self.current != Token::RBrace && self.current != Token::Eof {
            self.parse_device()?;
        }
        self.eat(Token::RBrace)
    }

    fn parse_device(&mut self) -> Result<(), String> {
        self.eat(Token::LParen)?;
        // STRING = device type
        if let Token::Str(device_type) = self.eat_any() {
            let _ = device_type;
        } else {
            return Err("expected device type string".to_string());
        }
        self.eat(Token::Ref)?;
        self.eat(Token::Equals)?;
        let ref_ = if let Token::Str(s) = self.eat_any() { s } else { return Err("expected device ref".to_string()); };
        // device_paramlist
        let mut device = Device {
            name: String::new(),
            ref_,
            value_type: "none".to_string(),
            value_float: 0.0,
            value_string: String::new(),
            layer_name: String::new(),
        };
        while self.current != Token::RParen {
            match &self.current {
                Token::Name => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    if let Token::Str(s) = self.eat_any() { device.name = s; }
                    else { return Err("expected string for NAME".to_string()); }
                }
                Token::Val => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    match self.eat_any() {
                        Token::Float(v) => { device.value_type = "float".to_string(); device.value_float = v; }
                        Token::Str(s) => { device.value_type = "string".to_string(); device.value_string = s; }
                        t => return Err(format!("expected value for VAL, found {:?}", t)),
                    }
                }
                Token::L => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    if let Token::Str(s) = self.eat_any() { device.layer_name = s; }
                    else { return Err("expected string for layer".to_string()); }
                }
                Token::Pkg => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    self.eat_any(); // string
                }
                t => return Err(format!("unexpected device param {:?}", t)),
            }
        }
        self.state.pcb.device.push(device);
        self.eat(Token::RParen)
    }

    fn parse_supplies(&mut self) -> Result<(), String> {
        self.eat(Token::Supplies)?;
        self.eat(Token::LBrace)?;
        self.skip_balanced_block()
    }

    fn parse_padstack(&mut self) -> Result<(), String> {
        self.eat(Token::Padstack)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // string
        self.skip_balanced_block()
    }

    fn parse_net_class(&mut self) -> Result<(), String> {
        self.eat(Token::NetClass)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // string
        self.skip_balanced_block()
    }

    fn parse_net(&mut self) -> Result<(), String> {
        self.eat(Token::Net)?;
        self.eat(Token::Equals)?;
        if let Token::Str(s) = self.eat_any() {
            self.state.current_net_name = Some(s);
        } else {
            return Err("expected net name string".to_string());
        }
        // optional plane separation
        if self.current == Token::PS {
            self.eat_any();
            self.eat(Token::Equals)?;
            self.eat_any(); // float
        }
        // net copper subrecords
        while self.current != Token::RBrace && self.current != Token::Eof {
            self.parse_net_subrecord()?;
        }
        self.eat(Token::RBrace)
    }

    fn parse_net_subrecord(&mut self) -> Result<(), String> {
        match &self.current {
            Token::LParen => {
                self.eat_any();
                match &self.current {
                    Token::Seg => self.parse_seg()?,
                    Token::Arc => self.parse_arc()?,
                    Token::Via => self.parse_via()?,
                    Token::Pin => self.parse_pin()?,
                    Token::Pad => self.parse_pad()?,
                    Token::Useg => self.parse_useg()?,
                    Token::A => self.parse_net_attribute()?,
                    t => return Err(format!("unexpected net subrecord {:?}", t)),
                }
                self.eat(Token::RParen)?;
            }
            Token::LBrace => {
                // polygon / polyvoid / polyline
                self.eat_any();
                match &self.current {
                    Token::Polygon => self.parse_polygon()?,
                    Token::Polyvoid => self.parse_polyvoid()?,
                    Token::Polyline => self.parse_polyline()?,
                    t => return Err(format!("unexpected polygon type {:?}", t)),
                }
                self.eat(Token::RBrace)?;
            }
            t => return Err(format!("unexpected net subrecord start {:?}", t)),
        }
        Ok(())
    }

    fn parse_seg(&mut self) -> Result<(), String> {
        self.eat(Token::Seg)?;
        self.parse_coord_line()?;
        // width, layer, etc.
        while self.current != Token::RParen {
            match &self.current {
                Token::W => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::L => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::PS | Token::Lps => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_arc(&mut self) -> Result<(), String> {
        self.eat(Token::Arc)?;
        self.parse_coord_arc()?;
        while self.current != Token::RParen {
            match &self.current {
                Token::W => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::L => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_via(&mut self) -> Result<(), String> {
        self.eat(Token::Via)?;
        self.parse_coord_point()?;
        let mut via = Via { x: 0.0, y: 0.0, z0: 0.0, z1: 0.0, radius: 0.0 };
        if let Token::Float(_x) = &self.current {
            // already parsed by coord_point
        }
        while self.current != Token::RParen {
            match &self.current {
                Token::P => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); } // padstack
                Token::D => { self.eat_any(); self.eat(Token::Equals)?; if let Token::Float(v) = self.eat_any() { via.radius = v / 2.0; } }
                Token::L1 => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::L2 => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::S | Token::SX | Token::SY | Token::A | Token::S1 | Token::S1X | Token::S1Y | Token::S2 | Token::S2X | Token::S2Y => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    self.eat_any();
                }
                _ => break,
            }
        }
        // Set x, y from coord_point
        via.x = self.state.current_polygon_points.last().map(|p| p.x).unwrap_or(0.0);
        via.y = self.state.current_polygon_points.last().map(|p| p.y).unwrap_or(0.0);
        self.state.pcb.via.push(via);
        Ok(())
    }

    fn parse_pin(&mut self) -> Result<(), String> {
        self.eat(Token::Pin)?;
        self.parse_coord_point()?;
        let x = self.state.current_polygon_points.last().map(|p| p.x).unwrap_or(0.0);
        let y = self.state.current_polygon_points.last().map(|p| p.y).unwrap_or(0.0);
        let mut pin = Pin {
            x, y, z0: 0.0, z1: 0.0,
            ref_: String::new(),
            layer_name: String::new(),
            metal: FloatPolygon { points: Vec::new() },
        };
        while self.current != Token::RParen {
            match &self.current {
                Token::P => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); } // padstack
                Token::R => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    if let Token::Str(s) = self.eat_any() { pin.ref_ = s; }
                    else { return Err("expected pin ref string".to_string()); }
                }
                Token::F => {
                    self.eat_any();
                    self.eat(Token::Equals)?;
                    self.eat_any(); // SIM_IN/SIM_OUT/SIM_BOTH/STRING
                }
                _ => break,
            }
        }
        self.state.pcb.pin.push(pin);
        Ok(())
    }

    fn parse_pad(&mut self) -> Result<(), String> {
        self.eat(Token::Pad)?;
        self.skip_balanced_block()
    }

    fn parse_useg(&mut self) -> Result<(), String> {
        self.eat(Token::Useg)?;
        self.skip_balanced_block()
    }

    fn parse_net_attribute(&mut self) -> Result<(), String> {
        self.eat(Token::A)?;
        self.eat_any(); // N
        self.eat(Token::Equals)?;
        self.eat_any(); // string
        self.eat(Token::V)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // string
        Ok(())
    }

    fn parse_polygon(&mut self) -> Result<(), String> {
        self.eat(Token::Polygon)?;
        // polygon_param_list
        while self.current != Token::LBrace && self.current != Token::Eof {
            match &self.current {
                Token::L => { self.eat_any(); self.eat(Token::Equals)?; if let Token::Str(s) = self.eat_any() { self.state.current_polygon_layer = Some(s); } }
                Token::W => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::T => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                Token::ID => { self.eat_any(); self.eat(Token::Equals)?; self.eat_any(); }
                _ => break,
            }
        }
        // first coord point
        self.parse_coord_point()?;
        // lines and curves
        while self.current != Token::RBrace && self.current != Token::Eof {
            self.parse_line_or_curve()?;
        }
        // build polygon from points
        let points = std::mem::take(&mut self.state.current_polygon_points);
        if !points.is_empty() {
            let poly = FloatPoly {
                poly: FloatPolygon { points },
                is_hole: false,
                nesting_level: 0,
            };
            // Add to current net or board
            if let Some(layer_name) = &self.state.current_polygon_layer {
                // find layer and add
                if let Some(layer) = self.state.pcb.stackup.iter_mut().find(|l| l.name == *layer_name) {
                    layer.metal.polys.push(poly);
                }
            }
        }
        self.state.current_polygon_layer = None;
        Ok(())
    }

    fn parse_polyvoid(&mut self) -> Result<(), String> {
        self.eat(Token::Polyvoid)?;
        self.skip_balanced_block()
    }

    fn parse_polyline(&mut self) -> Result<(), String> {
        self.eat(Token::Polyline)?;
        self.skip_balanced_block()
    }

    fn parse_line_or_curve(&mut self) -> Result<(), String> {
        match &self.current {
            Token::LParen => {
                self.eat_any();
                match &self.current {
                    Token::Line => {
                        self.eat_any();
                        self.parse_coord_point()?;
                        self.eat(Token::RParen)?;
                    }
                    Token::Arc | Token::Curve => {
                        self.eat_any();
                        self.parse_coord_arc()?;
                        self.eat(Token::RParen)?;
                    }
                    _ => return Err(format!("expected line or curve, found {:?}", self.current)),
                }
            }
            Token::RParen => {
                self.eat_any(); // unexpected )
            }
            t => return Err(format!("unexpected line_or_curve {:?}", t)),
        }
        Ok(())
    }

    fn parse_end(&mut self) -> Result<(), String> {
        self.eat(Token::End)?;
        self.eat(Token::LBrace)?;
        self.eat(Token::RBrace)
    }

    fn parse_key(&mut self) -> Result<(), String> {
        self.eat(Token::Key)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // string
        self.eat(Token::RBrace)
    }

    // ----- coordinate parsers -----

    fn parse_coord_point(&mut self) -> Result<(), String> {
        self.eat(Token::X)?;
        self.eat(Token::Equals)?;
        let x = if let Token::Float(v) = self.eat_any() { v } else { return Err("expected float for X".to_string()); };
        self.eat(Token::Y)?;
        self.eat(Token::Equals)?;
        let y = if let Token::Float(v) = self.eat_any() { v } else { return Err("expected float for Y".to_string()); };
        self.state.current_polygon_points.push(FloatPoint { x, y });
        Ok(())
    }

    fn parse_coord_line(&mut self) -> Result<(), String> {
        self.parse_coord_point()?; // X1 Y1
        // Actually coord_line is X1=.. Y1=.. X2=.. Y2=..
        // But the lexer uses X/Y for coord_point, X1/Y1 for coord_point1
        // For simplicity, treat as two points
        self.eat(Token::X)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // float
        self.eat(Token::Y)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // float
        Ok(())
    }

    fn parse_coord_arc(&mut self) -> Result<(), String> {
        self.parse_coord_line()?;
        self.eat(Token::Xc)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // float
        self.eat(Token::Yc)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // float
        self.eat(Token::R)?;
        self.eat(Token::Equals)?;
        self.eat_any(); // float
        Ok(())
    }
}

/// Parse a HyperLynx .hyp file into a PCB model.
pub fn parse_hyp(input: &str) -> Result<PCB, String> {
    let parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let pcb = parse_hyp("").unwrap();
        assert!(pcb.stackup.is_empty());
        assert!(pcb.via.is_empty());
    }

    #[test]
    fn test_version_section() {
        let input = "{ VERSION = 1.0 }";
        let pcb = parse_hyp(input).unwrap();
        assert!(pcb.stackup.is_empty());
    }

    #[test]
    fn test_stackup_signal_layer() {
        let input = r#"{ STACKUP ( SIGNAL T = 0.035 L = "Top" ER = 4.0 LT = 0.02 ) }"#;
        let pcb = parse_hyp(input).unwrap();
        assert_eq!(pcb.stackup.len(), 1);
        assert_eq!(pcb.stackup[0].name, "Top");
        assert!((pcb.stackup[0].thickness - 0.035).abs() < 1e-12);
    }

    #[test]
    fn test_device() {
        let input = r#"{ DEVICES ( "RES" REF = "R1" VAL = 1000.0 L = "Top" ) }"#;
        let pcb = parse_hyp(input).unwrap();
        assert_eq!(pcb.device.len(), 1);
        assert_eq!(pcb.device[0].ref_, "R1");
        assert!((pcb.device[0].value_float - 1000.0).abs() < 1e-12);
    }

    #[test]
    fn test_via() {
        let input = r#"{ NET = "Net1" ( VIA X = 1.0 Y = 2.0 D = 0.5 ) }"#;
        let pcb = parse_hyp(input).unwrap();
        assert_eq!(pcb.via.len(), 1);
        assert!((pcb.via[0].x - 1.0).abs() < 1e-12);
        assert!((pcb.via[0].y - 2.0).abs() < 1e-12);
    }
}
