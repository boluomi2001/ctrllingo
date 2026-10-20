//! 当前高亮候选的手动异步译文。

use qingjian_core::{Language, Translation};

use super::Composed;
use crate::dispatch::Router;

pub(crate) struct SentenceTranslation {
    pub(crate) text: String,

    pub(crate) language: Language,

    pub(crate) sequence: u64,

    pub(crate) translation: Option<Translation>,

    pub(crate) finished: bool,
}

impl SentenceTranslation {
    pub(crate) fn pending(text: String, language: Language, sequence: u64) -> Self {
        Self {
            text,
            language,
            sequence,
            translation: None,
            finished: false,
        }
    }
}

impl Router {
    pub(in crate::dispatch) fn invalidate_sentence_translation(&mut self) {
        self.cancel_pending_sentence_translation();
        self.engine.cancel_sentence_translation();
        if let Some(Composed::Candidates {
            translation_first,
            sentence_translation,
            ..
        }) = self.composed.as_mut()
        {
            *translation_first = false;
            *sentence_translation = None;
        }
    }

    pub(in crate::dispatch) fn refresh_sentence_translation(&mut self) {
        let current = match &self.composed {
            Some(Composed::Candidates { layout, .. }) => layout
                .candidate(self.highlight)
                .map(|candidate| (candidate.text.clone(), self.engine.learning_language())),
            _ => None,
        };
        let Some(Composed::Candidates {
            translation_first,
            sentence_translation,
            ..
        }) = self.composed.as_mut()
        else {
            return;
        };
        if sentence_translation.as_ref().is_none_or(|annotation| {
            current.as_ref().is_some_and(|(text, language)| {
                text == &annotation.text && *language == annotation.language
            })
        }) {
            return;
        }
        self.engine.cancel_sentence_translation();
        *sentence_translation = None;
        *translation_first = false;
    }

    pub(in crate::dispatch) fn highlighted_has_translation(&self) -> bool {
        let Some(Composed::Candidates { layout, .. }) = &self.composed else {
            return false;
        };
        let Some(candidate) = layout.candidate(self.highlight).cloned() else {
            return false;
        };
        let mut candidates = qingjian_core::CandidateList {
            items: vec![candidate],
        };
        self.engine.annotate(&mut candidates);
        self.annotate_sentence_translation(&mut candidates);
        candidates.items[0]
            .first_translation(self.engine.learning_language())
            .is_some()
    }

    pub(in crate::dispatch) fn request_highlighted_sentence_translation(&mut self) -> bool {
        let language = self.engine.learning_language();
        if !self.config.sentence_translation || language == Language::Chinese {
            return false;
        }
        let Some(Composed::Candidates { layout, .. }) = &self.composed else {
            return false;
        };
        let Some(candidate) = layout.candidate(self.highlight) else {
            return false;
        };
        let text = candidate.text.clone();
        self.engine.cancel_composition_prediction();
        let Some(sequence) = self.engine.request_candidate_translation(&text) else {
            return false;
        };
        if let Some(Composed::Candidates {
            sentence_translation,
            ..
        }) = self.composed.as_mut()
        {
            *sentence_translation = Some(SentenceTranslation::pending(text, language, sequence));
        }
        true
    }

    pub(in crate::dispatch) fn sentence_translation_failed(&self) -> bool {
        matches!(
            &self.composed,
            Some(Composed::Candidates {
                sentence_translation: Some(annotation),
                ..
            }) if annotation.finished && annotation.translation.is_none()
        )
    }

    pub(in crate::dispatch) fn cancel_pending_sentence_translation(&mut self) {
        let pending = matches!(
            &self.composed,
            Some(Composed::Candidates {
                sentence_translation: Some(annotation),
                ..
            }) if !annotation.finished
        );
        if pending {
            self.engine.cancel_sentence_translation();
            if let Some(Composed::Candidates {
                sentence_translation,
                ..
            }) = self.composed.as_mut()
            {
                *sentence_translation = None;
            }
        }
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
            Some("整句翻译失败；按 Ctrl 重试，或继续中文输入".to_owned())
        } else {
            Some("整句翻译中…".to_owned())
        }
    }
}
