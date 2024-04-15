//! TODO

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric0, char, hex_digit1, i32, u32},
    combinator::{map, map_res},
    error::Error,
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareEvent {
    UUDFEvent(UUDFEvent),
    UUDFPEvent(UUDFPEvent),
}

impl FromStr for HardwareEvent {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match alt((
            map(parse_uudf_event, |e| HardwareEvent::UUDFEvent(e)),
            map(parse_uudfp_event, |e| HardwareEvent::UUDFPEvent(e)),
        ))(s)
        .finish()
        {
            Ok((_remaining, event)) => Ok(event),
            Err(Error { input, code }) => Err(Error {
                input: input.to_owned(),
                code,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UUDFPEvent {
    instance_id: u64,
}

impl FromStr for UUDFPEvent {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_uudfp_event(s).finish() {
            Ok((_remaining, event)) => Ok(event),
            Err(Error { input, code }) => Err(Error {
                input: input.to_owned(),
                code,
            }),
        }
    }
}

/// The various data found in a UUDF event that comes over UART from the
/// u-blox antenna board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UUDFEvent {
    /// The ID of the tag whose data is being reported
    tag_id: u64,
    /// Signal strength
    rssi: i32,
    /// Azimuth to tag
    angle_1: i32,
    /// Elevation to tag
    angle_2: i32,
    /// u-blox won't say what they use this for...maybe curing cancer.
    reserved: i32,
    /// TODO what does it mean "channel", is this which Bluetooth channel?
    /// Completely unclear from the u-blox documentation.
    channel: u32,
    /// The ID of the antenna
    anchor_id: u64,
    /// The user can configure this with `AT+UDFCFG` tag 2
    user_defined: String,
    /// A timestamp in milliseconds since the listener block was powered on
    timestamp: u32,
    /// What event this is. The first reading is 1, then the next one is 2
    /// and so on.
    sequence: u32,
}

impl FromStr for UUDFEvent {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_uudf_event(s).finish() {
            Ok((_remaining, event)) => Ok(event),
            Err(Error { input, code }) => Err(Error {
                input: input.to_owned(),
                code,
            }),
        }
    }
}

fn parse_id(s: &str) -> IResult<&str, u64> {
    map_res(hex_digit1, |d: &str| {
        if d.len() == 12 {
            u64::from_str_radix(&d, 16)
        } else {
            u64::from_str_radix("hey", 0)
        }
    })(s)
}

fn parse_quoted_id(s: &str) -> IResult<&str, u64> {
    delimited(char('\"'), parse_id, char('\"'))(s)
}

fn parse_quoted_string(s: &str) -> IResult<&str, String> {
    map(
        delimited(char('\"'), alphanumeric0, char('\"')),
        |cs: &str| cs.to_owned(),
    )(s)
}

fn parse_uudf_event(s: &str) -> IResult<&str, UUDFEvent> {
    map(
        tuple((
            preceded(tag("+UUDF:"), parse_id),
            preceded(char(','), i32),
            preceded(char(','), i32),
            preceded(char(','), i32),
            preceded(char(','), i32),
            preceded(char(','), u32),
            preceded(char(','), parse_quoted_id),
            preceded(char(','), parse_quoted_string),
            preceded(char(','), u32),
            preceded(char(','), u32),
        )),
        |(
            instance_id,
            rssi,
            angle_1,
            angle_2,
            reserved,
            channel,
            anchor_id,
            user_defined,
            timestamp,
            sequence,
        )| UUDFEvent {
            tag_id: instance_id,
            rssi,
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

fn parse_uudfp_event(s: &str) -> IResult<&str, UUDFPEvent> {
    map(
        tuple((
            preceded(tag("+UUDFP:"), parse_id),
            preceded(char(','), hex_digit1),
        )),
        |(instance_id, _other_hex)| UUDFPEvent { instance_id },
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uudf_test_1() {
        let s = "+UUDF:CCF9578E0D8A,-42,20,0,-43,37,\"CCF9578E0D89\",\"\",15869,23";

        let res = UUDFEvent::from_str(s).unwrap();

        assert_eq!(
            res,
            UUDFEvent {
                tag_id: 0xCCF9578E0D8A,
                rssi: -42,
                angle_1: 20,
                angle_2: 0,
                reserved: -43,
                channel: 37,
                anchor_id: 0xCCF9578E0D89,
                user_defined: "".to_owned(),
                timestamp: 15869,
                sequence: 23,
            }
        );
    }

    #[test]
    fn uudf_test_2() {
        let s = "+UUDF:CCF9578E0D8B,-41,10,4,-42,38,\"CCF9578E0D89\",\"\",15892,24";

        let res = UUDFEvent::from_str(s).unwrap();

        assert_eq!(
            res,
            UUDFEvent {
                tag_id: 0xCCF9578E0D8B,
                rssi: -41,
                angle_1: 10,
                angle_2: 4,
                reserved: -42,
                channel: 38,
                anchor_id: 0xCCF9578E0D89,
                user_defined: "".to_owned(),
                timestamp: 15892,
                sequence: 24,
            }
        );
    }

    #[test]
    fn uudf_test_3() {
        let s = "+UUDF:CCF9578E0D8A,-42,-10,2,-43,39,\"CCF9578E0D89\",\"\",15921,25";

        let res = UUDFEvent::from_str(s).unwrap();

        assert_eq!(
            res,
            UUDFEvent {
                tag_id: 0xCCF9578E0D8A,
                rssi: -42,
                angle_1: -10,
                angle_2: 2,
                reserved: -43,
                channel: 39,
                anchor_id: 0xCCF9578E0D89,
                user_defined: "".to_owned(),
                timestamp: 15921,
                sequence: 25,
            }
        );
    }

    #[test]
    fn uufdp_test() {
        let s = "+UUDFP:6C3DEBAFAEE4,19FF1500000050F80C0065000900052A0D001F000000D0030000";

        let res = UUDFPEvent::from_str(s).unwrap();
        assert_eq!(
            res,
            UUDFPEvent {
                instance_id: 0x6C3DEBAFAEE4,
            }
        );
    }

    #[test]
    fn hardware_event_uudf_test_1() {
        let s = "+UUDF:CCF9578E0D8A,-42,20,0,-43,37,\"CCF9578E0D89\",\"\",15869,23";

        let res = HardwareEvent::from_str(s).unwrap();
        assert_eq!(
            res,
            HardwareEvent::UUDFEvent(UUDFEvent {
                tag_id: 0xCCF9578E0D8A,
                rssi: -42,
                angle_1: 20,
                angle_2: 0,
                reserved: -43,
                channel: 37,
                anchor_id: 0xCCF9578E0D89,
                user_defined: "".to_owned(),
                timestamp: 15869,
                sequence: 23,
            })
        );
    }

    #[test]
    fn hardware_event_uufdp_test() {
        let s = "+UUDFP:6C3DEBAFAEE4,19FF1500000050F80C0065000900052A0D001F000000D0030000";

        let res = HardwareEvent::from_str(s).unwrap();
        assert_eq!(
            res,
            HardwareEvent::UUDFPEvent(UUDFPEvent {
                instance_id: 0x6C3DEBAFAEE4,
            })
        );
    }
}
