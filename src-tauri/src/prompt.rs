/// 文本类型分类
#[derive(Debug, Clone, PartialEq)]
pub enum TextType {
    /// 单个英文单词（仅含字母，无空格/标点/数字）
    Word,
    /// 短语、句子、段落或非纯英文内容
    Sentence,
}

/// 判定文本类型：单个单词 vs 句子/短语
///
/// 判定规则：
/// - 去除首尾空白后，仅包含 ASCII 字母（a-z, A-Z），且长度不超过 50 → Word
/// - 否则 → Sentence
pub fn detect_text_type(text: &str) -> TextType {
    let trimmed = text.trim();

    if trimmed.is_empty() || trimmed.len() > 50 {
        return TextType::Sentence;
    }

    if trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        TextType::Word
    } else {
        TextType::Sentence
    }
}

/// 根据文本类型和配置构建对应的 prompt
pub fn build_prompt(text: &str, word_prompt: &str, sentence_prompt: &str) -> String {
    let template = match detect_text_type(text) {
        TextType::Word => word_prompt,
        TextType::Sentence => sentence_prompt,
    };
    template.replace("${text}", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_single_word() {
        assert_eq!(detect_text_type("hello"), TextType::Word);
        assert_eq!(detect_text_type("World"), TextType::Word);
        assert_eq!(detect_text_type("a"), TextType::Word);
    }

    #[test]
    fn test_detect_sentence() {
        assert_eq!(detect_text_type("hello world"), TextType::Sentence);
        assert_eq!(detect_text_type("hello, world"), TextType::Sentence);
        assert_eq!(detect_text_type("hello123"), TextType::Sentence);
        assert_eq!(detect_text_type(""), TextType::Sentence);
        assert_eq!(detect_text_type("你好"), TextType::Sentence);
        assert_eq!(detect_text_type("don't"), TextType::Sentence);
    }

    #[test]
    fn test_build_prompt_word() {
        let word_prompt = "词典释义：${text}";
        let sentence_prompt = "翻译：${text}";
        assert_eq!(build_prompt("hello", word_prompt, sentence_prompt), "词典释义：hello");
    }

    #[test]
    fn test_build_prompt_sentence() {
        let word_prompt = "词典释义：${text}";
        let sentence_prompt = "翻译：${text}";
        assert_eq!(build_prompt("hello world", word_prompt, sentence_prompt), "翻译：hello world");
    }
}
