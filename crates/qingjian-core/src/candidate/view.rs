//! 译文优先的展示副本；学习与拼音消耗始终使用原候选。

use super::{Candidate, Language, Sense, Translation};

impl Candidate {
    pub fn first_translation(&self, language: Language) -> Option<&str> {
        if language == Language::Chinese {
            return None;
        }
        self.translation
            .as_ref()
            .filter(|translation| translation.language == language)
            .and_then(|translation| translation.senses().first())
            .map(|sense| sense.text.as_str())
            .filter(|text| !text.trim().is_empty())
    }

    pub fn first_english_translation(&self) -> Option<&str> {
        self.first_translation(Language::English)
    }

    pub fn translation_view(&self) -> Self {
        self.translation_view_for(Language::English)
    }

    pub fn translation_view_for(&self, language: Language) -> Self {
        if self.text.is_empty() {
            return self.clone();
        }
        let text = self
            .first_translation(language)
            .map_or_else(|| "暂无译文".to_owned(), str::to_owned);
        Self {
            text,
            reading: None,
            aux_code: None,
            translation: Some(Translation::new(
                Language::Chinese,
                vec![Sense {
                    text: self.text.clone(),
                    part_of_speech: None,
                    reading: None,
                    fresh: false,
                }],
            )),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CandidateKind;

    fn candidate(language: Language, text: &str) -> Candidate {
        Candidate {
            text: "计算机".to_owned(),
            kind: CandidateKind::Chinese,
            syllables: vec!["ji".to_owned(), "suan".to_owned(), "ji".to_owned()],
            reading: None,
            aux_code: None,
            translation: Some(Translation::new(
                language,
                vec![Sense {
                    text: text.to_owned(),
                    part_of_speech: None,
                    reading: None,
                    fresh: false,
                }],
            )),
        }
    }

    #[test]
    fn english_view_preserves_the_source_candidate_and_reading_scope() {
        let source = candidate(Language::English, "computer");
        let view = source.translation_view();
        assert_eq!(view.text, "computer");
        assert_eq!(view.translation.unwrap().senses()[0].text, "计算机");
        assert_eq!(view.syllables, source.syllables);
        assert_eq!(source.text, "计算机");
        assert_eq!(source.first_english_translation(), Some("computer"));
    }

    #[test]
    fn translations_require_a_matching_non_chinese_language_and_nonempty_text() {
        for language in [
            Language::English,
            Language::Japanese,
            Language::Spanish,
            Language::Vietnamese,
            Language::German,
        ] {
            let source = candidate(language, "translated");
            assert_eq!(source.first_translation(language), Some("translated"));
            assert_eq!(source.translation_view_for(language).text, "translated");
            assert_eq!(source.first_translation(Language::Chinese), None);
            assert_eq!(source.first_translation(other_language(language)), None);
            assert_eq!(candidate(language, "  ").first_translation(language), None);
        }
        let chinese = candidate(Language::Chinese, "电脑");
        assert_eq!(chinese.first_translation(Language::Chinese), None);
        assert_eq!(chinese.first_translation(Language::English), None);
    }

    fn other_language(language: Language) -> Language {
        if language == Language::English {
            Language::Japanese
        } else {
            Language::English
        }
    }
}
