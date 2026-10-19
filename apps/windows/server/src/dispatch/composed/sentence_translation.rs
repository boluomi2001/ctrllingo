//! 一条本地整句候选的异步译文。

use qingjian_core::{CandidateKind, Language, Translation};

use super::Composed;
use crate::dispatch::Router;

pub(crate) struct SentenceTranslation {
    pub(crate) text: String,

    pub(crate) sequence: u64,

    pub(crate) translation: Option<Translation>,

    pub(crate) finished: bool,
}

impl SentenceTranslation {
    pub(crate) fn pending(text: String, sequence: u64) -> Self {
        Self {
            text,
            sequence,
            translation: None,
            finished: false,
        }
    }
}

impl Router {
    pub(in crate::dispatch) fn refresh_sentence_translation(&mut self) {
        let Some(Composed::Candidates {
            layout,
            sentence_translation,
            ..
        }) = self.composed.as_mut()
        else {
            return;
        };
        let is_sentence = |kind| matches!(kind, CandidateKind::Sentence | CandidateKind::Generated);
        let text = (self.config.sentence_translation
            && self.engine.learning_language() != Language::Chinese)
            .then(|| {
                layout
                    .candidate(self.highlight)
                    .filter(|candidate| is_sentence(candidate.kind))
                    .or_else(|| {
                        layout
                            .local()
                            .iter()
                            .find(|candidate| is_sentence(candidate.kind))
                    })
                    .map(|candidate| candidate.text.clone())
            })
            .flatten();
        if sentence_translation
            .as_ref()
            .map(|annotation| &annotation.text)
            == text.as_ref()
        {
            return;
        }
        self.engine.cancel_sentence_translation();
        *sentence_translation = text.and_then(|text| {
            self.engine
                .request_sentence_translation(&text)
                .map(|sequence| SentenceTranslation::pending(text, sequence))
        });
    }

    pub(super) fn sentence_translation_notice(&self) -> Option<String> {
        let Some(Composed::Candidates {
            sentence_translation: Some(annotation),
            ..
        }) = &self.composed
        else {
            return None;
        };
        if annotation.translation.is_some() {
            None
        } else if annotation.finished {
            Some("整句翻译未返回可用结果；可继续中文输入".to_owned())
        } else {
            Some("整句翻译中…".to_owned())
        }
    }
}
