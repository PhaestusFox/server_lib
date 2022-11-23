use serde::{Serialize, Deserialize};
use bevy_reflect::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
#[reflect_value(Serialize)]
pub struct Date(pub(crate) u32);
impl std::str::FromStr for Date {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[cfg(feature="yew")]
        web_sys::console::log_1(&s.into());
        let mut segs = if s.contains("-") {s.split('-')} else {s.split('/')};
        let year = if let Some(v) = segs.next() {match v.parse() {
            Ok(v) => v,
            Err(_) => return Err("Failed to parse year")
        }} else {return Err("No Year Seg");};
        let month = if let Some(v) = segs.next() {match v.parse() {
            Ok(v) => v,
            Err(_) => return Err("Failed to parse month")
        }} else {return Err("No Month Seg");};
        let day = if let Some(v) = segs.next() {match v.parse() {
            Ok(v) => v,
            Err(_) => return Err("Failed to parse day")
        }} else {return Err("No day Seg");};
        Ok(Date::new_ymd(year, month, day))
    }
}

impl Date {
    pub fn new_ymd(year: i16, month: u8, day: u8) -> Date {
        assert!(day <= if month == 2 {29} else {days_in_month(month)} && day > 0);
        assert!(month > 0 && month <= 12);
        let mut val = (year as u32) << 4;
        val += month as u32;
        val <<= 5;
        val += day as u32;
        Date(val << 7)
    }

    pub fn year(&self) -> i16 {
        (self.0 >> 16) as i16
    }

    pub fn month(&self) -> u8 {
        ((self.0 >> 12) & 0b1111) as u8
    }

    pub fn day(&self) -> u8 {
        ((self.0 >> 7) & 0b11111) as u8
    }

    pub fn next(&self) -> Date {
        let mut day = self.day();
        let mut month = self.month();
        let mut year = self.year();
        day += 1;
        if is_leap_year(year) && month == 2 && day > 29 {
            day = 1;
            month += 1;
        } else if day > days_in_month(month) && !(is_leap_year(year) && month == 2) {
            day = 1;
            month += 1;
        }
        if month > 12 {
            month = 1;
            year += 1;
        }
        Date::new_ymd(year, month, day)
    }

    pub fn prev(&self) -> Date {
        let mut day = self.day();
        let mut month = self.month();
        let mut year = self.year();
        day -= 1;
        if day == 0 {
            month -= 1;
            if is_leap_year(year) && month == 2 {
                day = 29;
            } else {
                day = days_in_month(month);
            }
        }
        if month == 0 {
            year -= 1;
            month = 12;
        }
        Date::new_ymd(year, month, day)
    }

    pub fn to_web_string(&self) -> String {
        format!("{}-{}-{}", self.year(), self.month(), self.day())
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.year())?;
        seq.serialize_element(&self.month())?;
        seq.serialize_element(&self.day())?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_seq(DateVisitor)
        //deserializer.deserialize_seq(3, DateVisitor)
    }
}
struct DateVisitor;
impl<'de> serde::de::Visitor<'de> for DateVisitor {
    type Value = Date;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expected date tuple")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let year = if let Some(y) = seq.next_element()? {y} else {return Err(serde::de::Error::missing_field("Year"));};
        let month = if let Some(y) = seq.next_element()? {y} else {return Err(serde::de::Error::missing_field("Month"));};
        let day = if let Some(y) = seq.next_element()? {y} else {return Err(serde::de::Error::missing_field("Day"));};
        Ok(Date::new_ymd(year, month, day))
    }
}

const fn days_in_month(month: u8) -> u8 {
    match month {
        00 => 31,
        01 => 31,
        02 => 28,
        03 => 31,
        04 => 30,
        05 => 31,
        06 => 30,
        07 => 31,
        08 => 31,
        09 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => panic!("only months 0-12 are real")
    }
}

const fn is_leap_year(year: i16) -> bool {
    if year % 100 == 0 {
        false
    } else if year % 4 == 0 {
        true
    } else {
        false
    }
}

#[cfg(feature="rocket")]
impl<'a> rocket::request::FromParam<'a> for Date {
    type Error = <Date as std::str::FromStr>::Err;
    fn from_param(str: &'a str) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        Date::from_str(str)
    }

}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn date_test() {
        let date = Date::new_ymd(2022, 09, 30);
        assert_eq!(date.year(), 2022);
        assert_eq!(date.day(), 30);
        assert_eq!(date.month(), 9);
        let date = Date::new_ymd(2030, 12, 11);
        assert_eq!(date.year(), 2030);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 11);
    }

    #[test]
    fn leap_year() {
        let date = Date::new_ymd(2020, 02, 29);
        let next = date.next();
        assert_eq!(next.day(), 1);
        assert_eq!(next.month(), 3);
        let prev = next.prev();
        assert_eq!(prev.day(), 29);
        assert_eq!(prev.month(), 2);
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}/{}", self.year(), self.month(), self.day()))
    }
}