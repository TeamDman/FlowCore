pub mod windy_error;
use eyre::Context;
use windows::Win32::System::Memory::GlobalLock;
use windows::Win32::System::Memory::GlobalSize;
use windows::Win32::System::Memory::GlobalUnlock;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Ole::*;
use windows::Win32::System::Memory::*;
use windows::Win32::UI::Shell::*;
use windy_error::WindyResult;

pub fn describe_clipboard_contents() -> WindyResult<String> {
    unsafe {
        OpenClipboard(None).wrap_err("Failed to open clipboard")?;

        let mut description = String::new();

        // Check for CF_HDROP format (file list)
        if IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() {
            let hdrop = GetClipboardData(CF_HDROP.0 as u32)?;
            if !hdrop.is_invalid() {
                let hdrop = HDROP(hdrop.0);
                let file_count = DragQueryFileW(hdrop, u32::MAX, None);

                description.push_str(&format!("Found {} files in clipboard:\n", file_count));

                for i in 0..file_count {
                    let mut buffer = vec![0u16; MAX_PATH as usize];
                    let len = DragQueryFileW(hdrop, i, Some(&mut buffer));
                    if len > 0 {
                        let path = OsString::from_wide(&buffer[..len as usize]);
                        description.push_str(&format!("- {}\n", path.to_string_lossy()));
                    }
                }
            }
        }

        // Check for additional formats
        let mut format = 0;
        loop {
            let next_format = EnumClipboardFormats(format);
            if next_format == 0 {
                let error = GetLastError();
                if error != ERROR_SUCCESS {
                    break; // Real error occurred
                }
                // No more formats to enumerate
                break;
            }
            format = next_format;
            let format_name = get_clipboard_format_name(format)?;
            description.push_str(&format!(
                "\nFormat: {} (0x{:X})\n",
                format_name, format
            ));

            // Try to get the content for this format
            if let Ok(handle) = GetClipboardData(format) {
                if !handle.is_invalid() {
                    let content = match format {
                        x if x == CF_TEXT.0 as u32 || x == CF_OEMTEXT.0 as u32 => {
                            let ptr = handle.0 as *const u8;
                            let mut len = 0;
                            while *ptr.add(len) != 0 {
                                len += 1;
                            }
                            let slice = std::slice::from_raw_parts(ptr, len);
                            String::from_utf8_lossy(slice).to_string()
                        }
                        x if x == CF_UNICODETEXT.0 as u32 => {
                            let ptr = handle.0 as *const u16;
                            let mut len = 0;
                            while *ptr.add(len) != 0 {
                                len += 1;
                            }
                            let slice = std::slice::from_raw_parts(ptr, len);
                            OsString::from_wide(slice).to_string_lossy().to_string()
                        }
                        _ => {
                            let size = GlobalSize(HGLOBAL(handle.0));
                            format!("[Binary data, {} bytes]", size)
                        }
                    };
                    description.push_str(&format!("Content: {}\n", content));
                }
            }
        }

        CloseClipboard()?;
        Ok(description)
    }
}

fn get_clipboard_format_name(format: u32) -> WindyResult<String> {
    unsafe {
        let mut buffer = vec![0u16; 256];
        let len = GetClipboardFormatNameW(format, &mut buffer);
        if len > 0 {
            Ok(OsString::from_wide(&buffer[..len as usize])
                .to_string_lossy()
                .to_string())
        } else {
            // Try to match against known clipboard formats
            let known_format = match format {
                x if x == CF_TEXT.0 as u32 => "CF_TEXT",
                x if x == CF_BITMAP.0 as u32 => "CF_BITMAP",
                x if x == CF_METAFILEPICT.0 as u32 => "CF_METAFILEPICT",
                x if x == CF_SYLK.0 as u32 => "CF_SYLK",
                x if x == CF_DIF.0 as u32 => "CF_DIF",
                x if x == CF_TIFF.0 as u32 => "CF_TIFF",
                x if x == CF_OEMTEXT.0 as u32 => "CF_OEMTEXT",
                x if x == CF_DIB.0 as u32 => "CF_DIB",
                x if x == CF_PALETTE.0 as u32 => "CF_PALETTE",
                x if x == CF_PENDATA.0 as u32 => "CF_PENDATA",
                x if x == CF_RIFF.0 as u32 => "CF_RIFF",
                x if x == CF_WAVE.0 as u32 => "CF_WAVE",
                x if x == CF_UNICODETEXT.0 as u32 => "CF_UNICODETEXT",
                x if x == CF_ENHMETAFILE.0 as u32 => "CF_ENHMETAFILE",
                x if x == CF_HDROP.0 as u32 => "CF_HDROP",
                x if x == CF_LOCALE.0 as u32 => "CF_LOCALE",
                x if x == CF_DIBV5.0 as u32 => "CF_DIBV5",
                _ => return Err(eyre::eyre!("Failed to get clipboard format name for {}", format).into()),
            };
            Ok(known_format.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_describe_clipboard_contents() {
        let description = describe_clipboard_contents().unwrap();
        println!("{}", description);
    }
}
