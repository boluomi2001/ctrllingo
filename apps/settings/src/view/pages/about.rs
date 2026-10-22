//! 「关于」页：版本与链接，更新，许可，数据来源，隐私。

use std::sync::Arc;

use bamboo_slip::taffy::{
    AlignItems, Dimension, FlexDirection, LengthPercentage, NodeId, Rect, Size, Style,
};
use bamboo_slip::{
    BoxPaint, Button, Fill, FontSpec, FontWeight, RenderError, SettingRow, TextStyle, Ui, Visual,
    group, paragraph, section_title,
};

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::actions;

const LICENSE: &str = "自由软件，GPL-3.0-or-later 许可证：可以自由使用、修改与再分发，修改后分发须同样开源。官方渠道免费。";

/// 随包数据的来源与许可（与旧设置程序「关于」页一致）。
const SOURCES: [(&str, &str); 8] = [
    (
        "词库",
        "通用规范汉字表；现代汉语常用词表（liuxilu 校对版）；THUOCL（清华大学自然语言处理实验室，MIT）；读音取自 Unihan（Unicode License v3）。",
    ),
    (
        "语言模型",
        "中文维基百科（CC BY-SA 4.0）与 LCCC（清华大学 CoAI，MIT）语料统计。",
    ),
    ("释义表", "由大语言模型（DeepSeek）生成，青简自建。"),
    ("emoji", "Unicode CLDR annotations（Unicode License v3）。"),
    (
        "英文词表",
        "ESDB / SCOWL（© Kevin Atkinson，按其许可保留版权声明）；CSpell 词典（MIT）。",
    ),
    (
        "词汇等级",
        "The CEFR-J Wordlist Version 1.5（Yukio Tono，Tokyo University of Foreign Studies，cefr-j.org）；Octanove Vocabulary Profile C1/C2（CC BY-SA 4.0）；JLPT 词表（tanos.co.uk，CC BY；经 elzup/jlpt-word-list 整理，MIT）。",
    ),
    (
        "五笔码表",
        "86 五笔极点码表（sxjudya/rime-wubi86-jidian，Apache-2.0）；编码来自上游，词频由青简词库按词面回填。",
    ),
    (
        "笔画码表",
        "CNS11643 全字庫筆順資料（數位發展部，政府資料開放授權條款第 1 版 / OFL-1.1）；大陆笔顺按通用规范字表校正。",
    ),
];

const PRIVACY: &str = "本地拼音输入在电脑上处理。使用云联想或翻译时，相关文本、拼音或上下文会直接发送至你配置的 AI 服务商；费用与服务商隐私政策由你确认。API 密钥可能以明文保存在本机配置或环境变量中。输入日志只保存在本机，可在高级页关闭或清空。当前 Windows 版不连接青简更新服务，下载请查看本项目 GitHub Releases。译点基于 Qingjian，按 GPL-3.0-or-later 分发，不提供担保；源码、许可与数据来源见项目仓库及随包 NOTICE。";

pub fn about(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let mut out = vec![header(ui, page)?];
    let download =
        Button::new("查看 GitHub 发布").build(ui, theme, "download", Message::CheckUpdate)?;
    out.push(section_title(ui, theme, "版本与更新")?);
    let row = SettingRow::new("译点独立发布")
        .info("基于青简二次开发；本版通过 GitHub Releases 手动更新")
        .control(download)
        .build(ui, theme)?;
    out.push(group(ui, theme, &[row])?);

    out.push(section_title(ui, theme, "许可")?);
    let license = note(ui, page, None, LICENSE)?;
    out.push(group(ui, theme, &[license])?);

    let open = page.settings.sources_open;
    let toggle_button = Button::new(if open { "收起" } else { "展开" }).build(
        ui,
        theme,
        "sources",
        Message::ToggleSources,
    )?;
    let mut rows = vec![
        SettingRow::new("词库、模型、词表与码表")
            .control(toggle_button)
            .build(ui, theme)?,
    ];
    if open {
        for (title, text) in SOURCES {
            rows.push(note(ui, page, Some(title), text)?);
        }
    }
    out.push(section_title(ui, theme, "数据来源")?);
    out.push(group(ui, theme, &rows)?);

    out.push(section_title(ui, theme, "隐私")?);
    let privacy = note(ui, page, None, PRIVACY)?;
    out.push(group(ui, theme, &[privacy])?);
    Ok(out)
}

/// logo、「青简」、版本，右边官网与 GitHub。
fn header(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let theme = &page.theme;
    let mut brand = Vec::new();
    if let Some(logo) = &page.settings.logo {
        let height = page.px(44.0);
        let (width, natural) = (logo.width() as f32, logo.height() as f32);
        brand.push(ui.scene().node(
            Style {
                size: Size {
                    width: Dimension::length(height * width / natural),
                    height: Dimension::length(height),
                },
                ..Style::default()
            },
            Visual::Box(BoxPaint {
                fill: Some(Fill::Image {
                    pixmap: Arc::clone(logo),
                    slice: None,
                    pixels_per_px: natural / height,
                }),
                border: None,
                radius: 0.0,
            }),
            &[],
        )?);
    }
    let font = FontSpec {
        weight: FontWeight(700),
        family: page.settings.serif,
        ..FontSpec::new(22.0, 30.0)
    };
    let name = ui.scene().node(
        Style::default(),
        Visual::Text {
            text: "译点".to_owned(),
            style: TextStyle::new(font.scaled(page.scale), 22.0, theme.colors.text, 1.0),
        },
        &[],
    )?;
    let version = ui.scene().node(
        Style::default(),
        Visual::Text {
            text: format!("版本 {}", env!("CARGO_PKG_VERSION")),
            style: theme.text(theme.small, 400, theme.colors.secondary),
        },
        &[],
    )?;
    let titles = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            gap: page.gap(2.0),
            ..Style::default()
        },
        Visual::Group,
        &[name, version],
    )?;
    brand.push(titles);
    let brand = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: page.gap(14.0),
            flex_grow: 1.0,
            ..Style::default()
        },
        Visual::Group,
        &brand,
    )?;
    let links = actions(
        ui,
        page,
        &[
            ("官网", Action::OpenWebsite),
            ("GitHub", Action::OpenRepository),
        ],
    )?;
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            padding: Rect {
                left: LengthPercentage::length(page.px(4.0)),
                right: LengthPercentage::length(page.px(4.0)),
                top: LengthPercentage::length(page.px(4.0)),
                bottom: LengthPercentage::length(page.px(8.0)),
            },
            ..Style::default()
        },
        Visual::Group,
        &[brand, links],
    )
}

/// 卡片里的一段文字，可带一个小标题。
fn note(
    ui: &mut Ui<Message>,
    page: &Page,
    title: Option<&str>,
    text: &str,
) -> Result<NodeId, RenderError> {
    let theme = &page.theme;
    let mut children = Vec::new();
    if let Some(title) = title {
        children.push(ui.scene().node(
            Style::default(),
            Visual::Text {
                text: title.to_owned(),
                style: theme.text(theme.font.size, 500, theme.colors.text),
            },
            &[],
        )?);
    }
    children.push(paragraph(ui, theme, text, None)?);
    let space = LengthPercentage::length(page.px(12.0));
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            gap: page.gap(4.0),
            padding: Rect {
                top: space,
                bottom: space,
                left: LengthPercentage::length(0.0),
                right: LengthPercentage::length(0.0),
            },
            ..Style::default()
        },
        Visual::Group,
        &children,
    )
}
