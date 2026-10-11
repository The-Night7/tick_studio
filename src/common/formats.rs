use super::{Jolt, Offset, Point};

#[allow(clippy::wrong_self_convention)]
pub trait PointFormat {
    fn to_buffer(&self, points: &[Point]) -> Result<Vec<u8>, String>;
    fn from_buffer(&self, buffer: &[u8]) -> Result<Vec<Point>, String>;
    fn get_extensions(&self) -> Vec<&'static str>;
    fn capabilities(&self) -> (bool, bool);
    fn get_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct ModernFormat;
impl PointFormat for ModernFormat {
    fn to_buffer(&self, points: &[Point]) -> Result<Vec<u8>, String> {
        serde_json::to_vec(points).map_err(|e| format!("Failed to serialize points: {}", e))
    }

    fn from_buffer(&self, buffer: &[u8]) -> Result<Vec<Point>, String> {
        serde_json::from_slice(buffer).map_err(|e| format!("Failed to deserialize points: {}", e))
    }

    fn get_extensions(&self) -> Vec<&'static str> {
        vec!["json"]
    }

    fn get_name(&self) -> &'static str {
        "Modern"
    }

    fn capabilities(&self) -> (bool, bool) {
        (true, true)
    }
}

pub struct DartFormat;

struct DartPoint {
    time: i32,
    tick: i32,
    jolt: Jolt,
    low_health: bool,
}

#[derive(serde::Deserialize)]
struct RawDartPoint {
    #[serde(deserialize_with = "fix_time")]
    timecode: i32,
    #[serde(rename = "uptime")]
    tick: i32,
    #[serde(rename = "joltCode")]
    jolt_code: String,
}

impl<'de> serde::Deserialize<'de> for DartPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawDartPoint::deserialize(deserializer)?;
        let (jolt, low_health) =
            parse_jolt_code(&raw.jolt_code).map_err(serde::de::Error::custom)?;
        Ok(Self {
            time: raw.timecode,
            tick: raw.tick,
            jolt,
            low_health,
        })
    }
}

#[derive(serde::Deserialize)]
struct DartFile {
    jolts: Vec<DartPoint>,
}

fn fix_time<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;

    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid timecode format"));
    }

    let time_parts: Vec<&str> = parts[0].split(':').collect();
    if time_parts.len() != 3 {
        return Err(serde::de::Error::custom("Invalid timecode format"));
    }

    let hours: i32 = time_parts[0]
        .parse()
        .map_err(|_| serde::de::Error::custom("Invalid hours"))?;
    let minutes: i32 = time_parts[1]
        .parse()
        .map_err(|_| serde::de::Error::custom("Invalid minutes"))?;
    let seconds: i32 = time_parts[2]
        .parse()
        .map_err(|_| serde::de::Error::custom("Invalid seconds"))?;

    let frames: i32 = parts[1]
        .parse()
        .map_err(|_| serde::de::Error::custom("Invalid frames"))?;

    let tick = (hours * 60 * 60 * 20) + (minutes * 60 * 20) + (seconds * 20) + ((frames * 20) / 60);

    Ok(tick)
}

fn parse_jolt_code(s: &str) -> Result<(Jolt, bool), String> {
    let parts = s.split_whitespace().collect::<Vec<&str>>();
    if parts.len() != 2 {
        return Err("Invalid jolt format".to_string());
    }

    let (amount, low_health) = match parts[0]
        .strip_suffix('H')
        .or_else(|| parts[0].strip_suffix('h'))
    {
        Some(number) => (number, true),
        None => (parts[0], false),
    };
    let food_amount = amount
        .parse::<i32>()
        .map_err(|_| "Invalid food amount".to_string())?;

    let pattern: [Offset; 10] = parts[1]
        .chars()
        .map(|c| match c {
            '+' => Ok(Offset::Up),
            '-' => Ok(Offset::Down),
            '0' => Ok(Offset::Neutral),
            _ => Err("Invalid character in jolt pattern".to_string()),
        })
        .collect::<Result<Vec<Offset>, String>>()?
        .try_into()
        .map_err(|_| "Invalid jolt pattern length".to_string())?;

    Ok((Jolt { food_amount, pattern }, low_health))
}

impl From<DartPoint> for Point {
    fn from(point: DartPoint) -> Self {
        Self {
            tick: point.tick,
            time: point.time,
            jolt: point.jolt,
            low_health: point.low_health,
        }
    }
}

impl PointFormat for DartFormat {
    fn to_buffer(&self, _points: &[Point]) -> Result<Vec<u8>, String> {
        Err("Dart format export is not supported".to_string())
    }

    fn from_buffer(&self, buffer: &[u8]) -> Result<Vec<Point>, String> {
        let content = decode_text(buffer)?;
        let file: DartFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize points: {e}"))?;
        Ok(file.jolts.into_iter().map(Point::from).collect())
    }

    fn get_extensions(&self) -> Vec<&'static str> {
        vec!["json"]
    }

    fn get_name(&self) -> &'static str {
        "Dart"
    }

    fn capabilities(&self) -> (bool, bool) {
        (true, false)
    }
}

pub struct LegacyFormat;

impl LegacyFormat {
    fn parse_row(row: &str) -> Result<Point, String> {
        let parts = row.split([':', '|', ',']).collect::<Vec<&str>>();
        if parts.len() < 11 {
            return Err(format!("Invalid row format: {}", row));
        }

        let hour = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse hour: {}", e))?;
        let minute = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse minute: {}", e))?;
        let second = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse second: {}", e))?;
        let frame = parts[3]
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse frame: {}", e))?;
        let food_amount = parts[7]
            .parse::<i32>()
            .map_err(|e| format!("Failed to parse food amount: {}", e))?;
        let pattern_str = parts[8];
        let tick = parts[10]
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse tick: {}", e))?;

        let pattern = pattern_str
            .chars()
            .map(|c| match c {
                '+' => Offset::Up,
                '-' => Offset::Down,
                _ => Offset::Neutral,
            })
            .collect::<Vec<Offset>>();

        if pattern.len() != 10 {
            return Err(format!("Invalid pattern length: {}", pattern_str));
        }

        let time = (hour * 3600 + minute * 60 + second) * 20 + frame / 3;

        Ok(Point {
            tick: tick as i32,
            time: time as i32,
            jolt: Jolt {
                pattern: pattern.try_into().unwrap(),
                food_amount,
            },
            low_health: false,
        })
    }
}

impl PointFormat for LegacyFormat {
    fn to_buffer(&self, _points: &[Point]) -> Result<Vec<u8>, String> {
        Err("Legacy format serialization is not supported".to_string())
    }

    fn from_buffer(&self, buffer: &[u8]) -> Result<Vec<Point>, String> {
        let content = decode_text(buffer)?;

        let mut points = Vec::new();
        for line in content.lines() {
            match Self::parse_row(line) {
                Ok(point) => points.push(point),
                Err(e) => return Err(format!("Failed to parse line '{}': {}", line, e)),
            }
        }
        Ok(points)
    }

    fn get_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "csv"]
    }

    fn get_name(&self) -> &'static str {
        "Legacy"
    }

    fn capabilities(&self) -> (bool, bool) {
        (true, false)
    }
}

fn decode_text(buffer: &[u8]) -> Result<String, String> {
    let (encoding, _) = encoding_rs::Encoding::for_bom(buffer).unwrap_or((encoding_rs::UTF_8, 0));
    let (text, _, failed) = encoding.decode(buffer);
    if failed {
        return Err(format!("Failed to decode buffer as {}", encoding.name()));
    }
    Ok(text.strip_prefix('\u{FEFF}').unwrap_or(&text).to_owned())
}

pub const ALL_FORMATS: &[&dyn PointFormat] = &[&ModernFormat, &LegacyFormat, &DartFormat];
