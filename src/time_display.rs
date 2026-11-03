// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Fixed-offset output formatting. Evidence without a known UTC offset is never relabeled.
use chrono::{DateTime, Datelike, FixedOffset, SecondsFormat, Utc};
use std::fmt;
use std::io;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputOffset(i32);

impl FromStr for OutputOffset {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() == 6
            && matches!(bytes[0], b'+' | b'-')
            && bytes[3] == b':'
            && [1, 2, 4, 5].iter().all(|&i| bytes[i].is_ascii_digit())
        {
            let hours = i32::from(bytes[1] - b'0') * 10 + i32::from(bytes[2] - b'0');
            let minutes = i32::from(bytes[4] - b'0') * 10 + i32::from(bytes[5] - b'0');
            if hours < 24 && minutes < 60 {
                let sign = if bytes[0] == b'-' { -1 } else { 1 };
                return Ok(Self(sign * (hours * 3600 + minutes * 60)));
            }
        }
        Err(format!(
            "Invalid timezone offset {value:?}. Use a quoted signed HH:MM offset (hours 00–23, minutes 00–59), for example --timezone \"+09:00\" or -Z \"-01:00\"."
        ))
    }
}

impl fmt::Display for OutputOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { '-' } else { '+' };
        let minutes = self.0.unsigned_abs() / 60;
        write!(f, "{sign}{:02}:{:02}", minutes / 60, minutes % 60)
    }
}

impl OutputOffset {
    pub fn is_utc(self) -> bool {
        self.0 == 0
    }

    fn fixed(self) -> FixedOffset {
        FixedOffset::east_opt(self.0).expect("validated output offset")
    }

    pub fn now(self) -> String {
        Utc::now()
            .with_timezone(&self.fixed())
            .to_rfc3339_opts(SecondsFormat::Millis, self.is_utc())
    }

    /// Change only an explicit, valid RFC 3339 instant. Keep the original fractional digits;
    /// chrono's AutoSi would pad a seven-digit Windows FILETIME to nine digits.
    pub fn timestamp(self, value: &str) -> io::Result<Option<String>> {
        let bytes = value.as_bytes();
        if bytes.len() < 20 || bytes.get(10) != Some(&b'T') || value.ends_with("-00:00") {
            return Ok(None);
        }
        let Ok(parsed) = DateTime::parse_from_rfc3339(value) else {
            return Ok(None);
        };
        let displayed = parsed.with_timezone(&self.fixed());
        if !(0..=9999).contains(&displayed.year()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Output timezone moves a timestamp outside years 0000–9999",
            ));
        }
        let fraction = if bytes.get(19) == Some(&b'.') {
            let end = 20
                + bytes[20..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit())
                    .count();
            // Reject unsupported precision instead of silently truncating evidence.
            if end - 20 > 9 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Timestamp has more than nine fractional digits",
                ));
            }
            &value[19..end]
        } else {
            ""
        };
        let suffix = if self.is_utc() {
            "Z".into()
        } else {
            self.to_string()
        };
        Ok(Some(format!(
            "{}{fraction}{suffix}",
            displayed.format("%Y-%m-%dT%H:%M:%S")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_require_a_sign_and_two_digit_hours_and_minutes() {
        for input in ["+09:00", "-01:00", "+02:30", "+23:59", "-23:59", "+00:00"] {
            assert_eq!(input.parse::<OutputOffset>().unwrap().to_string(), input);
        }
        for input in [
            "JST",
            "UTC",
            "+9",
            "-1",
            "09:00",
            "+9:00",
            "+24:00",
            "-24:00",
            "+00:60",
            "+00:000",
            "+０９:００",
            " +09:00",
            "",
        ] {
            let error = input.parse::<OutputOffset>().unwrap_err();
            assert!(error.contains("--timezone \"+09:00\""), "{error}");
        }
    }

    #[test]
    fn windows_timestamp_precision_and_instant_survive_conversion() {
        let timestamp = "2026-10-20T23:46:38.2001668Z";
        let tokyo: OutputOffset = "+09:00".parse().unwrap();
        let converted = tokyo.timestamp(timestamp).unwrap().unwrap();
        assert_eq!(converted, "2026-10-21T08:46:38.2001668+09:00");
        assert_eq!(
            OutputOffset::default()
                .timestamp(&converted)
                .unwrap()
                .unwrap(),
            timestamp
        );
        for (old, new) in [
            (
                "2026-10-20T08:46:38.200166800Z",
                "2026-10-20T17:46:38.200166800+09:00",
            ),
            ("2026-10-20T08:46:38Z", "2026-10-20T17:46:38+09:00"),
        ] {
            assert_eq!(tokyo.timestamp(old).unwrap().unwrap(), new);
        }
        assert_eq!(
            "-01:00"
                .parse::<OutputOffset>()
                .unwrap()
                .timestamp("2026-10-20T00:00:00.1Z")
                .unwrap()
                .unwrap(),
            "2026-10-19T23:00:00.1-01:00"
        );
        assert_eq!(
            "+02:30"
                .parse::<OutputOffset>()
                .unwrap()
                .timestamp("2026-10-20T23:45:00Z")
                .unwrap()
                .unwrap(),
            "2026-10-21T02:15:00+02:30"
        );
        assert!(
            tokyo
                .timestamp("2026-10-20T08:46:38-00:00")
                .unwrap()
                .is_none()
        );
        for raw in ["2026-10-20 08:46:38", "2026-10-20", "unknown", "0"] {
            assert!(tokyo.timestamp(raw).unwrap().is_none());
        }
        assert!(tokyo.timestamp("9999-12-31T23:59:59Z").is_err());
        assert!(tokyo.timestamp("2026-10-20T08:46:38.1234567890Z").is_err());
    }
}
