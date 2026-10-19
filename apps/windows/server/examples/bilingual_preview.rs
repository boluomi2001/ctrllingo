//! 离线画出真实候选帧，不注册 TSF，也不连接翻译服务。

use std::path::PathBuf;

use qingjian_core::Language;
use qingjian_platform::protocol::{ClientMessage, Frame, KeyEvent, ServerMessage, SessionId};
use qingjian_render::{FontLibrary, Layout, Mode, Preedit, Renderer, Theme};
use qingjian_windows_server::{AssemblySpec, Router, RouterConfig, assembly};

#[path = "../src/ui/candidates/row.rs"]
mod row;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let output = root.join("target/bilingual-preview");
    std::fs::create_dir_all(&output)?;
    let engine = assembly::assemble(&AssemblySpec {
        glossary: Some((
            Language::English,
            root.join("data/generated/glossary-en.qj"),
        )),
        ..AssemblySpec::new(root.join("data/generated/dict.qj"))
    })?;
    let mut router = Router::new(engine, RouterConfig::default());
    let session = SessionId(100);
    router.handle(ClientMessage::OpenSession {
        session,
        app: Some("notepad.exe".to_owned()),
        protocol: qingjian_platform::protocol::PROTOCOL_VERSION,
    });
    let mut frame = Frame::default();
    for c in "jisuanji".chars() {
        frame = key(
            &mut router,
            session,
            KeyEvent::new(c.to_ascii_uppercase() as u32, Some(c), Default::default()),
        );
    }
    let mut renderer = Renderer::new(FontLibrary::system("zh-CN")?);
    for (name, frame) in [
        ("chinese-left", frame),
        (
            "english-left",
            key(
                &mut router,
                session,
                KeyEvent::new(KeyEvent::SWAP_TRANSLATION, None, Default::default()),
            ),
        ),
    ] {
        let preedit: String = frame
            .preedit
            .iter()
            .map(|segment| segment.text.as_str())
            .collect();
        let rendered_frame = qingjian_render::Frame {
            preedit: Some(Preedit::plain(&preedit, frame.cursor)),
            rows: frame
                .candidates
                .items
                .iter()
                .enumerate()
                .map(|(index, candidate)| row::from_candidate(index, candidate, false, true))
                .collect(),
            highlighted: Some(frame.highlight),
            footer: None,
            sentence: None,
            status: frame.notice,
            mode: Mode::default(),
            columns: 0,
            column_ems: Vec::new(),
        };
        renderer.forget();
        renderer
            .render(&rendered_frame, Layout::Vertical, &Theme::light(), 1.5)?
            .pixmap
            .save_png(output.join(format!("{name}.png")))?;
    }
    println!("{}", output.display());
    Ok(())
}

fn key(router: &mut Router, session: SessionId, event: KeyEvent) -> Frame {
    match router.handle(ClientMessage::Key { session, event }) {
        Some(ServerMessage::KeyResult { frame, .. }) => frame,
        other => panic!("expected key result: {other:?}"),
    }
}
