use std::io::Cursor;

use base64::Engine;
use xcap::Monitor;

/// Capture the primary monitor as a PNG and return it as a base64-encoded string.
pub fn capture_primary_screen() -> Result<String, Box<dyn std::error::Error>> {
    let monitors = Monitor::all()?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| Monitor::all().ok()?.into_iter().next())
        .ok_or("No monitors found")?;

    let image = monitor.capture_image()?;

    // Encode as PNG into a buffer
    let mut buf = Cursor::new(Vec::new());
    image.write_to(&mut buf, image::ImageFormat::Png)?;

    // Base64 encode
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());

    Ok(b64)
}
