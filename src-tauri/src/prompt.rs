use unicode_segmentation::UnicodeSegmentation;

/// 文本类型分类
#[derive(Debug, Clone, PartialEq)]
pub enum TextType {
    /// 单个单词（含连字符、撇号等，适用于多语言）
    Word,
    /// 短语、句子、段落或非纯英文内容
    Sentence,
}

/// 判定文本类型：单个单词 vs 句子/短语
///
/// 判定规则：
/// - 以 Unicode 标准（UAX #29）对文本进行分词
/// - 若包含字母的片段数量为 1 → Word
/// - 否则 → Sentence
pub fn detect_text_type(text: &str) -> TextType {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return TextType::Sentence;
    }

    let word_count = trimmed
        .split_word_bounds()
        .filter(|s| s.chars().any(|c| c.is_alphabetic()))
        .count();

    if word_count == 1 {
        TextType::Word
    } else {
        TextType::Sentence
    }
}

/// 根据文本类型选择对应的 prompt 模板（不替换 ${text}）
pub fn select_template<'a>(text: &str, word_prompt: &'a str, sentence_prompt: &'a str) -> &'a str {
    match detect_text_type(text) {
        TextType::Word => word_prompt,
        TextType::Sentence => sentence_prompt,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_single_word_multi_lang() {
        // 英文
        assert_eq!(detect_text_type("hello"), TextType::Word);
        assert_eq!(detect_text_type("World"), TextType::Word);
        assert_eq!(detect_text_type("a"), TextType::Word);
        // 法语（带撇号，Unicode 视作同一词）
        assert_eq!(detect_text_type("bonjour"), TextType::Word);
        assert_eq!(detect_text_type("l'homme"), TextType::Word);
        // 俄语
        assert_eq!(detect_text_type("привет"), TextType::Word);
        // 阿拉伯语
        assert_eq!(detect_text_type("سلام"), TextType::Word);
        // 数字+字母混合
        assert_eq!(detect_text_type("hello123"), TextType::Word);
        assert_eq!(detect_text_type("a1b2c3"), TextType::Word);
        // 带撇号/连字符的英文
        assert_eq!(detect_text_type("don't"), TextType::Word);
        // 去除首尾空白
        assert_eq!(detect_text_type("  hello  "), TextType::Word);
    }

    #[test]
    fn test_detect_sentence_multi_lang() {
        // 英文短语
        assert_eq!(detect_text_type("hello world"), TextType::Sentence);
        assert_eq!(detect_text_type("hello, world"), TextType::Sentence);
        assert_eq!(detect_text_type("  hello world  "), TextType::Sentence);
        // 中文（CJK 每字独立成词）
        assert_eq!(detect_text_type("你好"), TextType::Sentence);
        assert_eq!(detect_text_type("你好世界"), TextType::Sentence);
        // 法语短语
        assert_eq!(detect_text_type("bonjour le monde"), TextType::Sentence);
        // 俄语短语
        assert_eq!(detect_text_type("привет мир"), TextType::Sentence);
        // 阿拉伯语短语
        assert_eq!(detect_text_type("مرحبا بالعالم"), TextType::Sentence);
        // 边界情况
        assert_eq!(detect_text_type(""), TextType::Sentence);
        assert_eq!(detect_text_type("   "), TextType::Sentence);
    }

    #[test]
    fn test_select_template_word() {
        let word_prompt = "词典释义：";
        let sentence_prompt = "翻译：";
        assert_eq!(
            select_template("hello", word_prompt, sentence_prompt),
            "词典释义："
        );
    }

    #[test]
    fn test_select_template_sentence() {
        let word_prompt = "词典释义：";
        let sentence_prompt = "翻译：";
        assert_eq!(
            select_template("hello world", word_prompt, sentence_prompt),
            "翻译："
        );
    }
}
