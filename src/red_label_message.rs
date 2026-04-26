//! Source `mess0.src` message-glyph and message-vector assets.

use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelScoreDigitImage {
    pub label: String,
    pub digit: u8,
    pub address: u16,
    pub width: u8,
    pub height: u8,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelMessageGlyphImage {
    pub label: String,
    pub character: char,
    pub address: u16,
    pub width: u8,
    pub height: u8,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelMessage {
    pub label: String,
    pub vector_address: u16,
    pub words: Vec<String>,
}

pub fn red_label_score_digit_image(digit: u8) -> Result<&'static RedLabelScoreDigitImage, String> {
    red_label_score_digit_images()?
        .iter()
        .find(|entry| entry.digit == digit)
        .ok_or_else(|| format!("red-label score digit asset has no digit {digit}"))
}

pub fn red_label_message_glyph(
    character: char,
) -> Result<&'static RedLabelMessageGlyphImage, String> {
    red_label_message_glyphs()?
        .iter()
        .find(|entry| entry.character == character)
        .ok_or_else(|| format!("red-label message glyph asset has no `{character}` entry"))
}

pub fn red_label_message(label: &str) -> Result<&'static RedLabelMessage, String> {
    red_label_messages()?
        .iter()
        .find(|entry| entry.label == label)
        .ok_or_else(|| format!("red-label message asset has no `{label}` entry"))
}

pub fn red_label_score_digit_images() -> Result<&'static [RedLabelScoreDigitImage], String> {
    static SCORE_DIGITS: OnceLock<Result<Vec<RedLabelScoreDigitImage>, String>> = OnceLock::new();
    match SCORE_DIGITS.get_or_init(|| parse_score_digits(crate::assets::RED_LABEL_SCORE_DIGITS_TSV))
    {
        Ok(images) => Ok(images.as_slice()),
        Err(error) => Err(error.clone()),
    }
}

pub fn red_label_message_glyphs() -> Result<&'static [RedLabelMessageGlyphImage], String> {
    static MESSAGE_GLYPHS: OnceLock<Result<Vec<RedLabelMessageGlyphImage>, String>> =
        OnceLock::new();
    match MESSAGE_GLYPHS
        .get_or_init(|| parse_message_glyphs(crate::assets::RED_LABEL_MESSAGE_GLYPHS_TSV))
    {
        Ok(images) => Ok(images.as_slice()),
        Err(error) => Err(error.clone()),
    }
}

pub fn red_label_messages() -> Result<&'static [RedLabelMessage], String> {
    static MESSAGES: OnceLock<Result<Vec<RedLabelMessage>, String>> = OnceLock::new();
    match MESSAGES.get_or_init(|| parse_messages(crate::assets::RED_LABEL_MESSAGES_TSV)) {
        Ok(messages) => Ok(messages.as_slice()),
        Err(error) => Err(error.clone()),
    }
}

pub fn parse_score_digits(text: &'static str) -> Result<Vec<RedLabelScoreDigitImage>, String> {
    let mut lines = text.lines();
    match lines.next() {
        Some("label\tdigit\taddress\twidth\theight\tbytes\tsource") => {}
        _ => {
            return Err(String::from(
                "red-label score digit asset header is invalid",
            ));
        }
    }

    let mut images = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 7 {
            return Err(format!(
                "red-label score digit line {line_number} must have 7 columns"
            ));
        }
        let label = columns[0];
        if label.is_empty() {
            return Err(format!(
                "red-label score digit line {line_number} has an empty label"
            ));
        }
        if columns[6].is_empty() {
            return Err(format!(
                "red-label score digit line {line_number} has an empty source"
            ));
        }
        let digit = parse_asset_u8("score digit", columns[1], line_number)?;
        if digit > 9 {
            return Err(format!(
                "red-label score digit {digit} on line {line_number} exceeds 9"
            ));
        }
        if images
            .iter()
            .any(|entry: &RedLabelScoreDigitImage| entry.digit == digit)
        {
            return Err(format!(
                "red-label score digit line {line_number} duplicates digit {digit}"
            ));
        }
        let width = parse_asset_u8("score digit width", columns[3], line_number)?;
        let height = parse_asset_u8("score digit height", columns[4], line_number)?;
        let bytes = parse_hex_pairs("score digit bytes", columns[5], line_number)?;
        if bytes.len() != usize::from(width) * usize::from(height) {
            return Err(format!(
                "red-label score digit line {line_number} byte count does not match {width}x{height}"
            ));
        }
        images.push(RedLabelScoreDigitImage {
            label: String::from(label),
            digit,
            address: parse_asset_hex_u16("score digit address", columns[2], line_number)?,
            width,
            height,
            bytes,
        });
    }

    if images.len() != 10 {
        return Err(format!(
            "red-label score digit asset has {} records instead of 10",
            images.len()
        ));
    }
    images.sort_by_key(|image| image.digit);
    Ok(images)
}

pub fn parse_message_glyphs(text: &'static str) -> Result<Vec<RedLabelMessageGlyphImage>, String> {
    let mut lines = text.lines();
    match lines.next() {
        Some("label\tcharacter\taddress\twidth\theight\tbytes\tsource") => {}
        _ => {
            return Err(String::from(
                "red-label message glyph asset header is invalid",
            ));
        }
    }

    let mut images = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 7 {
            return Err(format!(
                "red-label message glyph line {line_number} must have 7 columns"
            ));
        }
        let label = columns[0];
        if label.is_empty() {
            return Err(format!(
                "red-label message glyph line {line_number} has an empty label"
            ));
        }
        if columns[6].is_empty() {
            return Err(format!(
                "red-label message glyph line {line_number} has an empty source"
            ));
        }
        let character = match columns[1] {
            "SPACE" => ' ',
            value => {
                let mut chars = value.chars();
                let character = chars.next().ok_or_else(|| {
                    format!("red-label message glyph line {line_number} has an empty character")
                })?;
                if chars.next().is_some() {
                    return Err(format!(
                        "red-label message glyph character `{value}` on line {line_number} must be one character"
                    ));
                }
                character
            }
        };
        if images
            .iter()
            .any(|entry: &RedLabelMessageGlyphImage| entry.character == character)
        {
            return Err(format!(
                "red-label message glyph line {line_number} duplicates `{}`",
                columns[1]
            ));
        }
        let width = parse_asset_u8("message glyph width", columns[3], line_number)?;
        let height = parse_asset_u8("message glyph height", columns[4], line_number)?;
        let bytes = parse_hex_pairs("message glyph bytes", columns[5], line_number)?;
        if bytes.len() != usize::from(width) * usize::from(height) {
            return Err(format!(
                "red-label message glyph line {line_number} byte count does not match {width}x{height}"
            ));
        }
        images.push(RedLabelMessageGlyphImage {
            label: String::from(label),
            character,
            address: parse_asset_hex_u16("message glyph address", columns[2], line_number)?,
            width,
            height,
            bytes,
        });
    }

    if images.is_empty() {
        return Err(String::from("red-label message glyph asset has no records"));
    }
    Ok(images)
}

pub fn parse_messages(text: &'static str) -> Result<Vec<RedLabelMessage>, String> {
    let mut lines = text.lines();
    match lines.next() {
        Some("label\tvector_address\twords\tsource") => {}
        _ => return Err(String::from("red-label message asset header is invalid")),
    }

    let mut messages = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 4 {
            return Err(format!(
                "red-label message line {line_number} must have 4 columns"
            ));
        }
        let label = columns[0];
        if label.is_empty() {
            return Err(format!(
                "red-label message line {line_number} has an empty label"
            ));
        }
        if columns[2].is_empty() {
            return Err(format!(
                "red-label message line {line_number} has empty words"
            ));
        }
        if columns[3].is_empty() {
            return Err(format!(
                "red-label message line {line_number} has an empty source"
            ));
        }
        if messages
            .iter()
            .any(|entry: &RedLabelMessage| entry.label == label)
        {
            return Err(format!(
                "red-label message line {line_number} duplicates `{label}`"
            ));
        }
        let words: Vec<String> = columns[2]
            .split(' ')
            .map(String::from)
            .filter(|word| !word.is_empty())
            .collect();
        if words.is_empty() {
            return Err(format!("red-label message line {line_number} has no words"));
        }

        messages.push(RedLabelMessage {
            label: String::from(label),
            vector_address: parse_asset_hex_u16("message vector address", columns[1], line_number)?,
            words,
        });
    }

    if messages.is_empty() {
        return Err(String::from("red-label message asset has no records"));
    }
    Ok(messages)
}

fn parse_hex_pairs(context: &str, value: &str, line_number: usize) -> Result<Vec<u8>, String> {
    if value.is_empty() || !value.len().is_multiple_of(2) {
        return Err(format!(
            "red-label {context} `{value}` on line {line_number} must contain hex pairs"
        ));
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let mut index = 0;
    while index < value.len() {
        let byte_text = &value[index..index + 2];
        let byte = u8::from_str_radix(byte_text, 16).map_err(|_| {
            format!("red-label {context} byte `{byte_text}` on line {line_number} is invalid")
        })?;
        bytes.push(byte);
        index += 2;
    }
    Ok(bytes)
}

fn parse_asset_hex_u16(context: &str, value: &str, line_number: usize) -> Result<u16, String> {
    let hex = value.strip_prefix("0x").ok_or_else(|| {
        format!("red-label {context} `{value}` on line {line_number} must start with 0x")
    })?;
    u16::from_str_radix(hex, 16)
        .map_err(|_| format!("red-label {context} `{value}` on line {line_number} is invalid"))
}

fn parse_asset_u8(context: &str, value: &str, line_number: usize) -> Result<u8, String> {
    value
        .parse::<u8>()
        .map_err(|_| format!("red-label {context} `{value}` on line {line_number} is invalid"))
}

#[cfg(test)]
mod tests {
    use super::{parse_message_glyphs, parse_messages, red_label_message, red_label_message_glyph};

    #[test]
    fn message_assets_cover_crom0_rom_test_words() {
        let glyphs = parse_message_glyphs(crate::assets::RED_LABEL_MESSAGE_GLYPHS_TSV)
            .expect("message glyphs parse");
        let messages =
            parse_messages(crate::assets::RED_LABEL_MESSAGES_TSV).expect("messages parse");

        assert!(glyphs.iter().any(|glyph| glyph.character == 'F'));
        assert!(glyphs.iter().any(|glyph| glyph.character == ':'));
        assert_eq!(
            red_label_message_glyph('H').expect("H glyph").address,
            0xC87F
        );
        assert_eq!(red_label_message_glyph('I').expect("I glyph").width, 2);
        assert_eq!(
            red_label_message_glyph('R').expect("R glyph").address,
            0xC96F
        );
        assert!(messages.iter().any(|message| message.label == "VROMFL"));
        assert_eq!(
            red_label_message("VALROM")
                .expect("VALROM message")
                .words
                .as_slice(),
            &[
                String::from("ALL"),
                String::from("ROMS"),
                String::from("OK")
            ]
        );
        assert_eq!(
            red_label_message("VINS1")
                .expect("VINS1 message")
                .words
                .as_slice(),
            &[
                String::from("PRESS"),
                String::from("ADVANCE"),
                String::from("WITH"),
                String::from("SWITCH"),
                String::from("SET"),
                String::from("FOR:")
            ]
        );
    }
}
