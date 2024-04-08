use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric0, char, i32, one_of, u32},
    combinator::map,
    error::Error,
    multi::count,
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UUDFEvent {
    instance_id: String,
    rssi_first_polarization: i32,
    angle_1: i32,
    angle_2: i32,
    reserved: i32,
    channel: u32,
    anchor_id: String,
    user_defined: String,
    timestamp: u32,
    sequence: u32,
}

fn parse_id(s: &str) -> IResult<&str, String> {
    map(
        count(one_of("0123456789ABCDEFabcdef"), 12),
        |cs: Vec<char>| cs.into_iter().map(|c| c.to_ascii_uppercase()).collect(),
    )(s)
}

fn parse_quoted_id(s: &str) -> IResult<&str, String> {
    delimited(char('\"'), parse_id, char('\"'))(s)
}

fn parse_string(s: &str) -> IResult<&str, String> {
    map(
        delimited(char('\"'), alphanumeric0, char('\"')),
        |cs: &str| cs.to_owned(),
    )(s)
}

fn parse_uudf_elevent(s: &str) -> IResult<&str, UUDFEvent> {
    map(
        tuple((
            preceded(tag("+UUDF:"), parse_id),
            preceded(tag(","), i32),
            preceded(tag(","), i32),
            preceded(tag(","), i32),
            preceded(tag(","), i32),
            preceded(tag(","), u32),
            preceded(tag(","), parse_quoted_id),
            preceded(tag(","), parse_string),
            preceded(tag(","), u32),
            preceded(tag(","), u32),
        )),
        |(
            instance_id,
            rssi_first_polarization,
            angle_1,
            angle_2,
            reserved,
            channel,
            anchor_id,
            user_defined,
            timestamp,
            sequence,
        )| UUDFEvent {
            instance_id,
            rssi_first_polarization,
            angle_1,
            angle_2,
            reserved,
            channel,
            anchor_id,
            user_defined,
            timestamp,
            sequence,
        },
    )(s)
}

impl FromStr for UUDFEvent {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_uudf_elevent(s).finish() {
            Ok((_remaining, event)) => Ok(event),
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let s = "+UUDF:CCF9578E0D8A,-42,20,0,-43,37,\"CCF9578E0D89\",\"\",15869,23";

        let (leftover, res) = parse_uudf_elevent(s).unwrap();

        assert_eq!(leftover, "");
        assert_eq!(
            res,
            UUDFEvent {
                instance_id: "CCF9578E0D8A".to_owned(),
                rssi_first_polarization: -42,
                angle_1: 20,
                angle_2: 0,
                reserved: -43,
                channel: 37,
                anchor_id: "CCF9578E0D89".to_owned(),
                user_defined: "".to_owned(),
                timestamp: 15869,
                sequence: 23,
            }
        );
    }

    #[test]
    fn test_2() {
        let s = "+UUDF:CCF9578E0D8B,-41,10,4,-42,38,\"CCF9578E0D89\",\"\",15892,24";

        let (leftover, res) = parse_uudf_elevent(s).unwrap();

        assert_eq!(leftover, "");
        assert_eq!(
            res,
            UUDFEvent {
                instance_id: "CCF9578E0D8B".to_owned(),
                rssi_first_polarization: -41,
                angle_1: 10,
                angle_2: 4,
                reserved: -42,
                channel: 38,
                anchor_id: "CCF9578E0D89".to_owned(),
                user_defined: "".to_owned(),
                timestamp: 15892,
                sequence: 24,
            }
        );
    }

    #[test]
    fn test_3() {
        let s = "+UUDF:CCF9578E0D8A,-42,-10,2,-43,39,\"CCF9578E0D89\",\"\",15921,25";

        let (leftover, res) = parse_uudf_elevent(s).unwrap();

        assert_eq!(leftover, "");
        assert_eq!(
            res,
            UUDFEvent {
                instance_id: "CCF9578E0D8A".to_owned(),
                rssi_first_polarization: -42,
                angle_1: -10,
                angle_2: 2,
                reserved: -43,
                channel: 39,
                anchor_id: "CCF9578E0D89".to_owned(),
                user_defined: "".to_owned(),
                timestamp: 15921,
                sequence: 25,
            }
        );
    }
}
