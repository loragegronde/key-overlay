//! CSS colour parsing used by persisted key styles.

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Rgba {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 1.0,
    };

    pub fn to_egui(self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(
            self.r,
            self.g,
            self.b,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    pub fn to_css(self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            (self.a * 1000.0).round() / 1000.0
        )
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

pub fn parse_color(input: &str) -> Rgba {
    let value = input.trim().to_lowercase();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(hex).unwrap_or(Rgba::WHITE);
    }
    if let Some(inner) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<f32> = inner
            .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse().ok())
            .collect();
        if parts.len() >= 3 {
            return Rgba {
                r: parts[0].round().clamp(0.0, 255.0) as u8,
                g: parts[1].round().clamp(0.0, 255.0) as u8,
                b: parts[2].round().clamp(0.0, 255.0) as u8,
                a: parts.get(3).copied().map(clamp01).unwrap_or(1.0),
            };
        }
    }
    Rgba::WHITE
}

fn parse_hex(hex: &str) -> Option<Rgba> {
    let expand = |c: u8| -> u8 { let v = c; (v << 4) | v };
    match hex.len() {
        3 => {
            let bytes = hex.as_bytes();
            Some(Rgba {
                r: expand(from_hex(bytes[0])?),
                g: expand(from_hex(bytes[1])?),
                b: expand(from_hex(bytes[2])?),
                a: 1.0,
            })
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = if hex.len() == 8 {
                u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0
            } else {
                1.0
            };
            Some(Rgba { r, g, b, a })
        }
        _ => None,
    }
}

fn from_hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

/// Opaque pressed fill: blend glow into background without dropping alpha.
pub fn pressed_fill(background: &str, glow: &str, amount: f32) -> Rgba {
    let base = parse_color(background);
    let accent = parse_color(glow);
    let t = clamp01(amount);
    Rgba {
        r: ((base.r as f32) * (1.0 - t) + (accent.r as f32) * t).round() as u8,
        g: ((base.g as f32) * (1.0 - t) + (accent.g as f32) * t).round() as u8,
        b: ((base.b as f32) * (1.0 - t) + (accent.b as f32) * t).round() as u8,
        a: base.a.max(0.9),
    }
}

pub fn to_hex(input: &str) -> String {
    let c = parse_color(input);
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

pub fn with_hex(input: &str, hex: &str) -> String {
    let a = parse_color(input).a;
    let mut c = parse_color(hex);
    c.a = a;
    c.to_css()
}
