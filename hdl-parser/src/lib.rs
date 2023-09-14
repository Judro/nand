mod error;
use error::HdlError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, not},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use nom_locate2::*;
use nom_supreme::{
    error::{ErrorTree, GenericErrorTree},
    final_parser::final_parser,
};

pub type LResult<'s, O> = IResult<Input<'s>, O, ErrorTree<Input<'s>>>;

pub type LChipName<'s> = Located<'s, ChipName<'s>>;

pub type LPinName<'s> = Located<'s, PinName<'s>>;

pub type LRhsConnector<'s> = Located<'s, RhsConnector<'s>>;

pub type LLhsConnector<'s> = Located<'s, LhsConnector<'s>>;

pub type LUnsigned<'s> = Located<'s, u32>;

#[derive(Debug)]
pub struct Chip<'s> {
    pub name: LChipName<'s>,
    pub in_decl: Vec<LPinName<'s>>,
    pub out_decl: Vec<LPinName<'s>>,
    pub parts: Vec<Part<'s>>,
}

#[derive(Debug, PartialEq)]
pub struct BusSlice<'s> {
    name: LPinName<'s>,
    slice: (LUnsigned<'s>, LUnsigned<'s>),
}

#[derive(Debug, PartialEq)]
pub enum RhsConnector<'s> {
    Bus(&'s str),
    Slice(BusSlice<'s>),
    Potential(bool),
}

#[derive(Debug, PartialEq)]
pub enum LhsConnector<'s> {
    Bus(&'s str),
    Slice(BusSlice<'s>),
}

#[derive(Debug, PartialEq)]
pub struct PinName<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Connection<'s> {
    lhs: LLhsConnector<'s>,
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

pub fn parse_hdl2(input: &str) -> Result<Chip, HdlError> {
    match parse_hdl(Input::new(input)) {
        Ok(c) => Ok(c),
        Err(GenericErrorTree::Base { location, kind }) => {
            panic!("can not convert location: {location}")
        }
        Err(GenericErrorTree::Stack { base, contexts }) => {
            todo!()
        }
        Err(GenericErrorTree::Alt(alts)) => {
            todo!()
        }
    }
}

pub fn parse_hdl(input: Input) -> Result<Chip, ErrorTree<Input>> {
    final_parser(chip)(input)
}

fn chip(input: Input) -> LResult<Chip<'_>> {
    map(pair(chip_head, chip_body), |(h, b)| Chip {
        name: h,
        in_decl: b.0,
        out_decl: b.1,
        parts: b.2,
    })(input)
}

fn identifier(input: Input) -> LResult<&str> {
    get_str(preceded(
        not(digit1),
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
    ))(input)
}

fn unsigned(input: Input) -> LResult<u32> {
    map_res(get_str(digit1), |u: &str| u.parse::<u32>())(input)
}

pub fn chip_head(input: Input) -> LResult<LChipName> {
    preceded(
        delimited(skip_white, tag("CHIP"), skip_white),
        locate(chip_name),
    )(input)
}

pub fn chip_body(input: Input) -> LResult<(Vec<LPinName>, Vec<LPinName>, Vec<Part>)> {
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

fn skip_white(input: Input) -> LResult<&str> {
    get_str(multispace0)(input)
}

fn pin_list(input: Input) -> LResult<Vec<LPinName>> {
    separated_list0(
        preceded(skip_white, char(',')),
        preceded(skip_white, locate(pin_name)),
    )(input)
}

fn pin_name(input: Input) -> LResult<PinName> {
    map(identifier, PinName)(input)
}

fn chip_name(input: Input) -> LResult<ChipName> {
    map(identifier, ChipName)(input)
}

fn pin_decl<P: Pin>(input: Input) -> LResult<Vec<LPinName>> {
    preceded(
        skip_white,
        delimited(tag(P::conn()), pin_list, preceded(skip_white, char(';'))),
    )(input)
}

fn bool(input: Input) -> LResult<bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

fn bus_slice(input: Input) -> LResult<BusSlice> {
    map(
        pair(
            locate(pin_name),
            delimited(
                preceded(skip_white, char('[')),
                alt((
                    separated_pair(
                        preceded(skip_white, locate(unsigned)),
                        preceded(skip_white, tag("..")),
                        preceded(skip_white, locate(unsigned)),
                    ),
                    preceded(skip_white, map(locate(unsigned), |u| (u, u))),
                )),
                preceded(skip_white, char(']')),
            ),
        ),
        |(n, bs)| BusSlice { name: n, slice: bs },
    )(input)
}

fn rhs_connector(input: Input) -> LResult<LRhsConnector> {
    locate(alt((
        map(bool, RhsConnector::Potential),
        map(bus_slice, RhsConnector::Slice),
        map(identifier, RhsConnector::Bus),
    )))(input)
}

fn lhs_connector(input: Input) -> LResult<LLhsConnector> {
    locate(alt((
        map(bus_slice, LhsConnector::Slice),
        map(identifier, LhsConnector::Bus),
    )))(input)
}

fn connection(input: Input) -> LResult<Connection> {
    map(
        separated_pair(
            preceded(skip_white, lhs_connector),
            preceded(skip_white, char('=')),
            preceded(skip_white, rhs_connector),
        ),
        |(l, r)| Connection { lhs: l, rhs: r },
    )(input)
}

fn connection_list(input: Input) -> LResult<ConnectionList> {
    map(
        separated_list0(preceded(skip_white, char(',')), connection),
        ConnectionList,
    )(input)
}

fn part(input: Input) -> LResult<Part> {
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
        let (rem, res) = identifier(Input::new("ALU_01!")).unwrap();
        assert_eq!(rem.into_fragment(), "!");
        assert_eq!(res, "ALU_01");
    }

    #[test]
    fn ident_1() {
        let r = identifier(Input::new("99ALU_01!"));
        assert!(r.is_err())
    }

    #[test]
    fn ident_2() {
        let (rem, res) = identifier(Input::new("_ALU_01 ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(res, "_ALU_01");
    }

    #[test]
    fn chip_head_0() {
        let (rem, res) = chip_head(Input::new("\tCHIP ALU_01!")).unwrap();
        assert_eq!(rem.into_fragment(), "!");
        assert_eq!(res, ChipName("ALU_01"));
        assert_eq!(res.span(), (6, 12))
    }
    #[test]
    fn chip_head_1() {
        let res = chip_head(Input::new("\tChip ALU_01!"));
        assert!(res.is_err())
    }

    #[test]
    fn chip_head_2() {
        let (rem, res) = chip_head(Input::new("CHIPALU_01!")).unwrap();
        assert_eq!(rem.into_fragment(), "!");
        assert_eq!(res, ChipName("ALU_01"));
    }

    #[test]
    fn body_0() {
        let (rem, _res) = chip_body(Input::new(" {\tIN;\nOUT; PARTS: } ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
    }

    #[test]
    #[should_panic]
    fn body_1() {
        let (_rem, _res) = chip_body(Input::new(" {  ")).unwrap();
    }

    #[test]
    fn pin_list_0() {
        let (rem, res) = pin_list(Input::new("a, b , c,d;")).unwrap();
        assert_eq!(rem.into_fragment(), ";");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    fn pin_list_1() {
        let (rem, res) = pin_list(Input::new("a, b  c,d;")).unwrap();
        assert_eq!(res, vec![PinName("a"), PinName("b")]);
        assert_eq!(rem.into_fragment(), "  c,d;");
    }

    #[test]
    fn input_pin_0() {
        let (rem, res) = pin_decl::<In>(Input::new("IN a, b , c,d; ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    #[should_panic]
    fn input_pin_1() {
        let (_rem, _res) = pin_decl::<In>(Input::new("IN a, b , c,d")).unwrap();
    }

    #[test]
    #[should_panic]
    fn input_pin_2() {
        let (_rem, _res) = pin_decl::<In>(Input::new("OUT a, b , c,d")).unwrap();
    }

    #[test]
    fn output_pin_0() {
        let (rem, res) = pin_decl::<Out>(Input::new("OUT a, b , c,d; ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(
            res,
            vec![PinName("a"), PinName("b"), PinName("c"), PinName("d")]
        );
    }

    #[test]
    fn bus_slice0() {
        let (rem, slice) = bus_slice(Input::new("a [ 22 ] ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(slice.name, PinName("a"));
        assert_eq!(slice.slice.0, 22);
        assert_eq!(slice.slice.1, 22);
    }

    #[test]
    fn bus_slice1() {
        let (rem, slice) = bus_slice(Input::new("a[ 1 .. 3 ] ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(slice.name, PinName("a"));
        assert_eq!(slice.slice.0, 1);
        assert_eq!(slice.slice.1, 3);
    }

    #[test]
    fn rhs_connector_0() {
        let (rem, res) = rhs_connector(Input::new("true ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(res, RhsConnector::Potential(true));
    }

    #[test]
    fn rhs_connector_1() {
        let (rem, res) = rhs_connector(Input::new("false ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(res, RhsConnector::Potential(false));
    }

    #[test]
    fn rhs_connector_2() {
        let (rem, res) = rhs_connector(Input::new("ab ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(res, RhsConnector::Bus("ab"));
    }

    #[test]
    fn rhs_connector_3() {
        let (rem, _res) = rhs_connector(Input::new("a[10..12] ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        // just check that it works
    }

    #[test]
    fn connection_0() {
        let (rem, Connection { lhs: a, rhs: b }) = connection(Input::new(" a = b ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(a, LhsConnector::Bus("a"));
        assert_eq!(b, RhsConnector::Bus("b"));
    }
    #[test]
    #[should_panic]
    fn connection_1() {
        let (_rem, _c) = connection(Input::new(" a = 1 ")).unwrap();
    }

    #[test]
    fn connection_list_0() {
        let (rem, ConnectionList(list)) =
            connection_list(Input::new(" a = b , c = false , d= in ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(list[0].lhs.get(), &LhsConnector::Bus("a"));
        assert_eq!(list[1].lhs.get(), &LhsConnector::Bus("c"));
        assert_eq!(list[2].lhs.get(), &LhsConnector::Bus("d"));
    }

    #[test]
    fn part_0() {
        let (rem, part) = part(Input::new("And ( a = a , b = false ) ; ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(part.name, ChipName("And"));
        assert_eq!(part.connections.0.len(), 2);
    }

    #[test]
    fn chip_0() {
        let (rem, chip) = chip(Input::new("CHIP And { \n IN a,b; OUT out; \n PARTS: \n Nand(a=a,b=b,out=c); \n Not(in=c,out=out); } ")).unwrap();
        assert_eq!(rem.into_fragment(), " ");
        assert_eq!(chip.name, ChipName("And"));
        assert_eq!(chip.in_decl, vec![PinName("a"), PinName("b")]);
        assert_eq!(chip.out_decl, vec![PinName("out")]);
        assert_eq!(chip.parts[0].name, ChipName("Nand"));
        assert_eq!(chip.parts[1].name, ChipName("Not"));
    }
}
