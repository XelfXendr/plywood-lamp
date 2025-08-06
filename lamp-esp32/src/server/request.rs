use chrono::{DateTime, Utc};
use embassy_time::Duration;
use httparse::Status;
use microjson::JSONValue;

use crate::{types::Color, server::parse_error::ParseError};

pub enum LedRequest {
    Set(Color, Duration),
    DaylightCycle(Color, DateTime<Utc>, [u64; 4]),
}

impl LedRequest {
    pub fn parse_http(buffer: &[u8]) -> Result<LedRequest, ParseError> {
        // parse HTTP headers
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let header_end = if let Status::Complete(n) = req.parse(&buffer)? {
            n
        } else {
            Err(httparse::Error::Status)?
        };

        // parse JSON body
        let body = core::str::from_utf8(&buffer[header_end..])?;
        let json = JSONValue::load(body);

        let request = match json.get_key_value("type")?.read_string()? {
            "set" => {
                /*
                expected format:
                {
                    type: "set",
                    color: [255, 244, 200]
                    duration: 10000
                }
                */
                let color = Self::parse_color(json.get_key_value("color")?)?;
                let duration =
                    Duration::from_millis(json.get_key_value("duration")?.read_integer()? as u64);

                LedRequest::Set(color, duration)
            }
            "cycle" => {
                /*
                expected format:
                {
                    type: "cycle",
                    on_color: [255, 244, 200],
                    current_time: "2014-11-28T21:00:09+09:00",
                    cycle_minutes: [540, 600, 1260, 1320]
                }
                */
                let on_color = Self::parse_color(json.get_key_value("on_color")?)?;
                let current_time: DateTime<Utc> =
                    json.get_key_value("current_time")?.read_string()?.parse()?;

                let mut minutes_iter = json.get_key_value("cycle_minutes")?.iter_array()?;
                let mut minutes: [u64; 4] = [0; 4];
                minutes.iter_mut().try_for_each(|m| -> Result<(), ParseError> {
                    *m = minutes_iter.next().ok_or(ParseError::ValueError)?.read_integer()? as u64;
                    Ok(())
                })?;
                
                // make sure minutes are ordered and in correct range
                for i in 0..3 {
                    if minutes[i] > minutes[i+1] {
                        Err(ParseError::ValueError)?
                    }
                }
                if minutes[3] > 24*60 {
                    Err(ParseError::ValueError)?
                }

                LedRequest::DaylightCycle(on_color, current_time, minutes)
            }
            _ => Err(ParseError::ValueError)?,
        };

        Ok(request)
    }

    fn parse_color(val: JSONValue) -> Result<Color, ParseError> {
        let mut iter = val.iter_array()?;
        let r = iter.next().ok_or(ParseError::ValueError)?.read_integer()? as u8;
        let g = iter.next().ok_or(ParseError::ValueError)?.read_integer()? as u8;
        let b = iter.next().ok_or(ParseError::ValueError)?.read_integer()? as u8;
        Ok(Color::new(r, g, b))
    }
}
