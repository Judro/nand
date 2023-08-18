use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, not},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use nom_locate::{locate, Located};

pub type LChipName<'s> = Located<ChipName<'s>, &'s str>;

pub type LPinName<'s> = Located<PinName<'s>, &'s str>;

pub type LRhsConnector<'s> = Located<RhsConnector<'s>, &'s str>;

pub struct Chip<'s> {
    pub name: LChipName<'s>,
    pub in_decl: Vec<LPinName<'s>>,
    pub out_decl: Vec<LPinName<'s>>,
    pub parts: Vec<Part<'s>>,
}

#[derive(Debug, PartialEq)]
pub enum RhsConnector<'s> {
    Bus(&'s str),
    Potential(bool),
}
#[derive(Debug, PartialEq)]
pub struct PinName<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Connection<'s> {
    lhs: LPinName<'s>,
    rhs: LRhsConnector<'s>,
}

#[derive(Debug, PartialEq)]
pub struct ConnectionList<'s>(Vec<Connection<'s>>);

#[derive(Debug, PartialEq)]
pub struct ChipName<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Part<'s> {
    name: LChipName<'s>,
    connections: ConnectionList<'s>,
}

pub fn chip(input: &str) -> IResult<&str, Chip<'_>> {
    map(pair(chip_head, chip_body), |(h, b)| Chip {
        name: h,
        in_decl: b.0,
        out_decl: b.1,
        parts: b.2,
    })(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    preceded(
        not(digit1),
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
    )(input)
}
pub fn chip_head(input: &str) -> IResult<&str, LChipName> {
    preceded(
        delimited(skip_white, tag("CHIP"), skip_white),
        locate(chip_name),
    )(input)
}

pub fn chip_body(input: &str) -> IResult<&str, (Vec<LPinName>, Vec<LPinName>, Vec<Part>)> {
    map(
        delimited(
            preceded(skip_white, char('{')),
            pair(
                tuple((pin_decl::<In>, pin_decl::<Out>)),
                preceded(preceded(skip_white, tag("PARTS:")), many0(part)),
            ),
            preceded(skip_white, char('}')),
        ),
        |(io, p)| (io.0, io.1, p),
    )(input)
}

fn skip_white(input: &str) -> IResult<&str, &str> {
    multispace0(input)
}

fn pin_list(input: &str) -> IResult<&str, Vec<LPinName>> {
    separated_list0(
        preceded(skip_white, char(',')),
        preceded(skip_white, locate(pin_name)),
    )(input)
}

fn pin_name(input: &str) -> IResult<&str, PinName> {
    map(identifier, |s| PinName(s))(input)
}

fn chip_name(input: &str) -> IResult<&str, ChipName> {
    map(identifier, |s| ChipName(s))(input)
}

fn pin_decl<P: Pin>(input: &str) -> IResult<&str, Vec<LPinName>> {
    preceded(
        skip_white,
        delimited(tag(P::conn()), pin_list, preceded(skip_white, char(';'))),
    )(input)
}

fn bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

fn rhs_connector(input: &str) -> IResult<&str, LRhsConnector> {
    locate(alt((
        map(bool, RhsConnector::Potential),
        map(identifier, RhsConnector::Bus),
    )))(input)
}

fn connection(input: &str) -> IResult<&str, Connection> {
    map(
        separated_pair(
            preceded(skip_white, locate(pin_name)),
            preceded(skip_white, char('=')),
            preceded(skip_white, rhs_connector),
        ),
        |(l, r)| Connection { lhs: l, rhs: r },
    )(input)
}

fn connection_list(input: &str) -> IResult<&str, ConnectionList> {
    map(
        separated_list0(preceded(skip_white, char(',')), connection),
        |c| ConnectionList(c),
    )(input)
}

fn part(input: &str) -> IResult<&str, Part> {
    map(
        terminated(
            pair(
                preceded(skip_white, locate(chip_name)),
                delimited(
                    preceded(skip_white, char('(')),
                    connection_list,
                    preceded(skip_white, char(')')),
                ),
            ),
            preceded(skip_white, char(';')),
        ),
        |p| Part {
            name: p.0,
            connections: p.1,
        },
    )(input)
}

struct In;
struct Out;

trait Pin {
    fn conn() -> &'static str;
}

impl Pin for In {
    fn conn() -> &'static str {
        "IN"
    }
}

impl Pin for Out {
    fn conn() -> &'static str {
        "OUT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ident_0() {
        let (rem, res) = identifier("ALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(res, "ALU_01");
    }

    #[test]
    #[should_panic]
    fn ident_1() {
        let (_rem, _res) = identifier("99ALU_01!").unwrap();
    }

    #[test]
    fn ident_2() {
        let (rem, res) = identifier("_ALU_01 ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, "_ALU_01");
    }

    #[test]
    fn chip_head_0() {
        let (rem, res) = chip_head("\tCHIP ALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(res, ChipName("ALU_01"));
    }
    #[test]
    #[should_panic]
    fn chip_head_1() {
        let (_rem, _res) = chip_head("\tChip ALU_01!").unwrap();
    }

    #[test]
    fn chip_head_2() {
        let (rem, res) = chip_head("CHIPALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(res, ChipName("ALU_01"));
    }

    #[test]
    fn body_0() {
        let (rem, _res) = chip_body(" {\tIN;\nOUT; PARTS: } ").unwrap();
        assert_eq!(rem, " ");
    }

    #[test]
    #[should_panic]
    fn body_1() {
        let (_rem, _res) = chip_body(" {  ").unwrap();
    }

    #[test]
    fn pin_list_0() {
        let (rem, res) = pin_list("a, b , c,d;").unwrap();
        assert_eq!(rem, ";");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    fn pin_list_1() {
        let (rem, res) = pin_list("a, b  c,d;").unwrap();
        assert_eq!(res, vec![PinName("a"), PinName("b")]);
        assert_eq!(rem, "  c,d;");
    }

    #[test]
    fn input_pin_0() {
        let (rem, res) = pin_decl::<In>("IN a, b , c,d; ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    #[should_panic]
    fn input_pin_1() {
        let (_rem, _res) = pin_decl::<In>("IN a, b , c,d").unwrap();
    }

    #[test]
    #[should_panic]
    fn input_pin_2() {
        let (_rem, _res) = pin_decl::<In>("OUT a, b , c,d").unwrap();
    }

    #[test]
    fn output_pin_0() {
        let (rem, res) = pin_decl::<Out>("OUT a, b , c,d; ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    fn rhs_connector_0() {
        let (rem, res) = rhs_connector("true ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Potential(true));
    }

    #[test]
    fn rhs_connector_1() {
        let (rem, res) = rhs_connector("false ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Potential(false));
    }

    #[test]
    fn rhs_connector_2() {
        let (rem, res) = rhs_connector("ab ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Bus("ab"));
    }

    #[test]
    fn connection_0() {
        let (rem, Connection { lhs: a, rhs: b }) = connection(" a = b ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(a, PinName("a"));
        assert_eq!(b, RhsConnector::Bus("b"));
    }
    #[test]
    #[should_panic]
    fn connection_1() {
        let (_rem, _c) = connection(" a = 1 ").unwrap();
    }

    #[test]
    fn connection_list_0() {
        let (rem, ConnectionList(list)) = connection_list(" a = b , c = false , d= in ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(list[0].lhs.get().0, "a");
        assert_eq!(list[1].lhs.get().0, "c");
        assert_eq!(list[2].lhs.get().0, "d");
    }

    #[test]
    fn part_0() {
        let (rem, part) = part("And ( a = a , b = false ) ; ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(part.name, ChipName("And"));
        assert_eq!(part.connections.0.len(), 2);
    }

    #[test]
    fn chip_0() {
        let (rem, chip) = chip("CHIP And { \n IN a,b; OUT out; \n PARTS: \n Nand(a=a,b=b,out=c); \n Not(in=c,out=out); } ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(chip.name, ChipName("And"));
        assert_eq!(chip.in_decl, vec![PinName("a"), PinName("b")]);
        assert_eq!(chip.out_decl, vec![PinName("out")]);
        assert_eq!(chip.parts[0].name, ChipName("Nand"));
        assert_eq!(chip.parts[1].name, ChipName("Not"));
    }
}
