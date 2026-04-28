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
/// - 去除首尾空白后，中间存在空格 → Sentence
/// - 否则 → Word
pub fn detect_text_type(text: &str) -> TextType {
    let trimmed = text.trim();

    if trimmed.contains(' ') {
        TextType::Sentence
    } else {
        TextType::Word
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
        assert_eq!(detect_text_type("  hello world  "), TextType::Sentence);
    }

    #[test]
    fn test_detect_word_with_non_alpha() {
        assert_eq!(detect_text_type("hello123"), TextType::Word);
        assert_eq!(detect_text_type("don't"), TextType::Word);
        assert_eq!(detect_text_type("你好"), TextType::Word);
        assert_eq!(detect_text_type(""), TextType::Word);
        assert_eq!(detect_text_type("  hello  "), TextType::Word);
    }

    #[test]
    fn test_build_prompt_word() {
        let word_prompt = "词典释义：${text}";
        let sentence_prompt = "翻译：${text}";
        assert_eq!(
            build_prompt("hello", word_prompt, sentence_prompt),
            "词典释义：hello"
        );
    }

    #[test]
    fn test_build_prompt_sentence() {
        let word_prompt = "词典释义：${text}";
        let sentence_prompt = "翻译：${text}";
        assert_eq!(
            build_prompt("hello world", word_prompt, sentence_prompt),
            "翻译：hello world"
        );
    }
}
