//! GDI 绘制费用卡片的各个几何元素与文本。

use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    Arc, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreatePen, CreateSolidBrush,
    DEFAULT_CHARSET, DT_CALCRECT, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK,
    DeleteObject, DrawTextW, Ellipse, FF_DONTCARE, FW_BOLD, HDC, LineTo, MoveToEx, OUT_TT_PRECIS,
    PS_SOLID, RoundRect, SelectObject, SetTextColor, VARIABLE_PITCH,
};
use windows::core::w;

use super::{ACCENT, BACKGROUND, CARD, EDGE, MUTED};
pub(super) fn draw_card(
    hdc: HDC,
    scale: impl Fn(i32) -> i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    let brush = unsafe { CreateSolidBrush(CARD) };
    let pen = unsafe { CreatePen(PS_SOLID, scale(1), EDGE) };
    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
    let old_pen = unsafe { SelectObject(hdc, pen.into()) };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(left),
            scale(top),
            scale(right),
            scale(bottom),
            scale(12),
            scale(12),
        );
    }
    unsafe {
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(pen.into());
        let _ = DeleteObject(brush.into());
    }
}

pub(super) fn draw_badge(
    hdc: HDC,
    scale: impl Fn(i32) -> i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    let brush = unsafe { CreateSolidBrush(COLORREF(0x00F1F3E1)) };
    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
    let old_pen = unsafe {
        SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN),
        )
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(left),
            scale(top),
            scale(right),
            scale(bottom),
            scale(12),
            scale(12),
        );
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(brush.into());
    }
}

pub(super) fn draw_icon_tile(hdc: HDC, scale: impl Fn(i32) -> i32, left: i32, top: i32) {
    let brush = unsafe { CreateSolidBrush(COLORREF(0x00F1F3E1)) };
    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
    let old_pen = unsafe {
        SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN),
        )
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(left),
            scale(top),
            scale(left + 40),
            scale(top + 40),
            scale(14),
            scale(14),
        );
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(brush.into());
    }
}

pub(super) fn draw_token_icon(hdc: HDC, scale: impl Fn(i32) -> i32) {
    let brush = unsafe { CreateSolidBrush(ACCENT) };
    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
    let old_pen = unsafe {
        SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN),
        )
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(34),
            scale(72),
            scale(58),
            scale(88),
            scale(3),
            scale(3),
        );
        let _ = Ellipse(hdc, scale(34), scale(67), scale(58), scale(79));
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(brush.into());
    }
    let row_pen = unsafe { CreatePen(PS_SOLID, scale(1), COLORREF(0x00F1F3E1)) };
    let old = unsafe { SelectObject(hdc, row_pen.into()) };
    unsafe {
        let _ = MoveToEx(hdc, scale(37), scale(79), None);
        let _ = LineTo(hdc, scale(55), scale(79));
        let _ = MoveToEx(hdc, scale(37), scale(84), None);
        let _ = LineTo(hdc, scale(55), scale(84));
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(row_pen.into());
    }
}

pub(super) fn draw_wallet_icon(hdc: HDC, scale: impl Fn(i32) -> i32) {
    let brush = unsafe { CreateSolidBrush(ACCENT) };
    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
    let old_pen = unsafe {
        SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN),
        )
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(33),
            scale(269),
            scale(59),
            scale(290),
            scale(4),
            scale(4),
        );
        let _ = RoundRect(
            hdc,
            scale(37),
            scale(274),
            scale(62),
            scale(286),
            scale(3),
            scale(3),
        );
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(brush.into());
    }
}

pub(super) fn draw_document_icon(hdc: HDC, scale: impl Fn(i32) -> i32, x: i32, y: i32) {
    let pen = unsafe { CreatePen(PS_SOLID, scale(2), ACCENT) };
    let old = unsafe { SelectObject(hdc, pen.into()) };
    let null_brush = unsafe {
        windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH)
    };
    let old_brush = unsafe { SelectObject(hdc, null_brush) };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(x),
            scale(y),
            scale(x + 16),
            scale(y + 20),
            scale(3),
            scale(3),
        );
        let _ = MoveToEx(hdc, scale(x + 4), scale(y + 6), None);
        let _ = LineTo(hdc, scale(x + 12), scale(y + 6));
        let _ = MoveToEx(hdc, scale(x + 4), scale(y + 10), None);
        let _ = LineTo(hdc, scale(x + 12), scale(y + 10));
        let _ = MoveToEx(hdc, scale(x + 4), scale(y + 14), None);
        let _ = LineTo(hdc, scale(x + 10), scale(y + 14));
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
}

pub(super) fn draw_globe_icon(hdc: HDC, scale: impl Fn(i32) -> i32) {
    let pen = unsafe { CreatePen(PS_SOLID, scale(1), MUTED) };
    let old = unsafe { SelectObject(hdc, pen.into()) };
    let null_brush = unsafe {
        windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH)
    };
    let old_brush = unsafe { SelectObject(hdc, null_brush) };
    unsafe {
        let _ = Ellipse(hdc, scale(28), scale(351), scale(62), scale(383));
        let _ = Ellipse(hdc, scale(38), scale(351), scale(52), scale(383));
        let _ = Arc(
            hdc,
            scale(32),
            scale(354),
            scale(58),
            scale(378),
            scale(32),
            scale(366),
            scale(58),
            scale(366),
        );
        let _ = Arc(
            hdc,
            scale(32),
            scale(356),
            scale(58),
            scale(380),
            scale(58),
            scale(368),
            scale(32),
            scale(368),
        );
        let _ = MoveToEx(hdc, scale(29), scale(367), None);
        let _ = LineTo(hdc, scale(61), scale(367));
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
}

pub(super) fn draw_settings_icon(hdc: HDC, scale: impl Fn(i32) -> i32) {
    let pen = unsafe { CreatePen(PS_SOLID, scale(2), ACCENT) };
    let old = unsafe { SelectObject(hdc, pen.into()) };
    let null_brush = unsafe {
        windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH)
    };
    let old_brush = unsafe { SelectObject(hdc, null_brush) };
    unsafe {
        let _ = Ellipse(hdc, scale(194), scale(394), scale(214), scale(414));
        let _ = MoveToEx(hdc, scale(204), scale(392), None);
        let _ = LineTo(hdc, scale(204), scale(397));
        let _ = MoveToEx(hdc, scale(204), scale(411), None);
        let _ = LineTo(hdc, scale(204), scale(416));
        let _ = MoveToEx(hdc, scale(192), scale(404), None);
        let _ = LineTo(hdc, scale(197), scale(404));
        let _ = MoveToEx(hdc, scale(211), scale(404), None);
        let _ = LineTo(hdc, scale(216), scale(404));
        let _ = MoveToEx(hdc, scale(195), scale(395), None);
        let _ = LineTo(hdc, scale(199), scale(399));
        let _ = MoveToEx(hdc, scale(209), scale(409), None);
        let _ = LineTo(hdc, scale(213), scale(413));
        let _ = MoveToEx(hdc, scale(195), scale(413), None);
        let _ = LineTo(hdc, scale(199), scale(409));
        let _ = MoveToEx(hdc, scale(209), scale(399), None);
        let _ = LineTo(hdc, scale(213), scale(395));
        let _ = Ellipse(hdc, scale(200), scale(400), scale(208), scale(408));
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
}

pub(super) fn draw_separator(hdc: HDC, scale: impl Fn(i32) -> i32, left: i32, y: i32, right: i32) {
    let pen = unsafe { CreatePen(PS_SOLID, scale(1), EDGE) };
    let old = unsafe { SelectObject(hdc, pen.into()) };
    unsafe {
        let _ = MoveToEx(hdc, scale(left), scale(y), None);
        let _ = LineTo(hdc, scale(right), scale(y));
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
}

pub(super) fn draw_separator_vertical(
    hdc: HDC,
    scale: impl Fn(i32) -> i32,
    x: i32,
    top: i32,
    bottom: i32,
) {
    let pen = unsafe { CreatePen(PS_SOLID, scale(1), EDGE) };
    let old = unsafe { SelectObject(hdc, pen.into()) };
    unsafe {
        let _ = MoveToEx(hdc, scale(x), scale(top), None);
        let _ = LineTo(hdc, scale(x), scale(bottom));
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
}

pub(super) fn draw_segment(
    hdc: HDC,
    scale_dpi: u32,
    scale: impl Fn(i32) -> i32,
    x: i32,
    label: &str,
    selected: bool,
) {
    let background = unsafe {
        CreateSolidBrush(if selected {
            COLORREF(0x00F1F3E1)
        } else {
            BACKGROUND
        })
    };
    let old_brush = unsafe { SelectObject(hdc, background.into()) };
    let old_pen = unsafe {
        SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN),
        )
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            scale(x),
            scale(10),
            scale(x + 48),
            scale(36),
            scale(10),
            scale(10),
        );
    }
    unsafe {
        let _ = SelectObject(hdc, old_pen);
        let _ = SelectObject(hdc, old_brush);
        let _ = DeleteObject(background.into());
    }
    draw_text(
        scale_dpi,
        hdc,
        label,
        (scale(x + 6), scale(15), scale(36)),
        false,
        if selected { ACCENT } else { MUTED },
    );
}

pub(super) fn draw_brand_icon(hdc: HDC, scale: impl Fn(i32) -> i32) {
    use windows::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DrawIconEx, LoadIconW};
    use windows::core::PCWSTR;
    if let Ok(icon) = unsafe {
        LoadIconW(
            Some(super::super::module_handle()),
            PCWSTR(core::ptr::without_provenance::<u16>(1)),
        )
    } {
        unsafe {
            let _ = DrawIconEx(
                hdc,
                scale(12),
                scale(10),
                icon,
                scale(26),
                scale(26),
                0,
                None,
                DI_NORMAL,
            );
        }
    }
}

pub(super) fn draw_text(
    scale_dpi: u32,
    hdc: HDC,
    text: &str,
    bounds: (i32, i32, i32),
    bold: bool,
    color: COLORREF,
) {
    draw_text_sized(scale_dpi, hdc, text, bounds, bold, color, 15);
}

pub(super) fn draw_text_sized(
    scale_dpi: u32,
    hdc: HDC,
    text: &str,
    bounds: (i32, i32, i32),
    bold: bool,
    color: COLORREF,
    font_size: i32,
) {
    if text.is_empty() {
        return;
    }
    let (x, y, width) = bounds;
    let height = (font_size * scale_dpi as i32 / 96).max(1);
    let font = unsafe {
        CreateFontW(
            -height,
            0,
            0,
            0,
            if bold { FW_BOLD.0 as i32 } else { 400 },
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_TT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Microsoft YaHei UI"),
        )
    };
    if font.is_invalid() {
        return;
    }
    let old = unsafe { SelectObject(hdc, font.into()) };
    unsafe {
        let _ = SetTextColor(hdc, color);
    }
    let mut rect = RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height * 2,
    };
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
    unsafe {
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }
}

pub(super) fn draw_wrapped(scale_dpi: u32, hdc: HDC, text: &str, rect: &mut RECT, color: COLORREF) {
    if text.is_empty() {
        return;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut chosen_font = None;
    for font_size in (10..=14).rev() {
        let height = (font_size * scale_dpi as i32 / 96).max(1);
        let font = unsafe {
            CreateFontW(
                -height,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_TT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
                w!("Microsoft YaHei UI"),
            )
        };
        if font.is_invalid() {
            continue;
        }
        let old = unsafe { SelectObject(hdc, font.into()) };
        let mut measured = *rect;
        unsafe {
            let _ = DrawTextW(
                hdc,
                &mut wide,
                &mut measured,
                DT_LEFT | DT_WORDBREAK | DT_NOPREFIX | DT_CALCRECT,
            );
            let _ = SelectObject(hdc, old);
        }
        if (measured.bottom <= rect.bottom && measured.right <= rect.right) || font_size == 10 {
            chosen_font = Some(font);
            break;
        }
        unsafe {
            let _ = DeleteObject(font.into());
        }
    }
    let Some(font) = chosen_font else {
        return;
    };
    let old = unsafe { SelectObject(hdc, font.into()) };
    unsafe {
        let _ = SetTextColor(hdc, color);
    }
    let mut draw_rect = *rect;
    unsafe {
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut draw_rect,
            DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
        );
    }
    unsafe {
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }
}
