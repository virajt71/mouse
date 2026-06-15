const CHOSEONG_LIST: &[char] = &[
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
];

const JUNGSEONG_LIST: &[char] = &[
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'
];

const JONGSEONG_LIST: &[char] = &[
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
];

#[derive(Debug, Clone, Default)]
pub struct HangulComposer {
    choseong: Option<char>,
    jungseong: Option<char>,
    jongseong: Option<char>,
}

pub enum HangulAction {
    Update { backspaces: usize, text: String },
    ResetAndType(char),
}

fn is_consonant(c: char) -> bool {
    c >= 'ㄱ' && c <= 'ㅎ'
}

fn is_vowel(c: char) -> bool {
    c >= 'ㅏ' && c <= 'ㅣ'
}

fn combine_vowels(v1: char, v2: char) -> Option<char> {
    match (v1, v2) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

fn combine_jongseongs(j1: char, j2: char) -> Option<char> {
    match (j1, j2) {
        ('ㄱ', 'ㅅ') => Some('ㄳ'),
        ('ㄴ', 'ㅈ') => Some('ㄵ'),
        ('ㄴ', 'ㅎ') => Some('ㄶ'),
        ('ㄹ', 'ㄱ') => Some('ㄺ'),
        ('ㄹ', 'ㅁ') => Some('ㄻ'),
        ('ㄹ', 'ㅂ') => Some('ㄼ'),
        ('ㄹ', 'ㅅ') => Some('ㄽ'),
        ('ㄹ', 'ㅌ') => Some('ㄾ'),
        ('ㄹ', 'ㅍ') => Some('ㄿ'),
        ('ㄹ', 'ㅎ') => Some('ㅀ'),
        ('ㅂ', 'ㅅ') => Some('ㅄ'),
        _ => None,
    }
}

fn decompose_jongseong(j: char) -> Option<(char, char)> {
    match j {
        'ㄳ' => Some(('ㄱ', 'ㅅ')),
        'ㄵ' => Some(('ㄴ', 'ㅈ')),
        'ㄶ' => Some(('ㄴ', 'ㅎ')),
        'ㄺ' => Some(('ㄹ', 'ㄱ')),
        'ㄻ' => Some(('ㄹ', 'ㅁ')),
        'ㄼ' => Some(('ㄹ', 'ㅂ')),
        'ㄽ' => Some(('ㄹ', 'ㅅ')),
        'ㄾ' => Some(('ㄹ', 'ㅌ')),
        'ㄿ' => Some(('ㄹ', 'ㅍ')),
        'ㅀ' => Some(('ㄹ', 'ㅎ')),
        'ㅄ' => Some(('ㅂ', 'ㅅ')),
        _ => None,
    }
}

fn make_syllable(cho: char, jung: char, jong: Option<char>) -> char {
    let cho_idx = CHOSEONG_LIST.iter().position(|&x| x == cho).unwrap_or(0);
    let jung_idx = JUNGSEONG_LIST.iter().position(|&x| x == jung).unwrap_or(0);
    let jong_idx = match jong {
        Some(j) => JONGSEONG_LIST.iter().position(|&x| x == j).unwrap_or(0),
        None => 0,
    };
    let unicode_val = 0xAC00 + ((cho_idx * 21) + jung_idx) * 28 + jong_idx;
    std::char::from_u32(unicode_val as u32).unwrap_or(' ')
}

impl HangulComposer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.choseong = None;
        self.jungseong = None;
        self.jongseong = None;
    }

    pub fn feed(&mut self, ch: char) -> HangulAction {
        if is_consonant(ch) {
            match (self.choseong, self.jungseong, self.jongseong) {
                (None, _, _) => {
                    self.choseong = Some(ch);
                    HangulAction::Update { backspaces: 0, text: ch.to_string() }
                }
                (Some(_cho), None, _) => {
                    // Two consonants back-to-back without vowel: commit previous and start new
                    self.reset();
                    self.choseong = Some(ch);
                    HangulAction::Update { backspaces: 0, text: ch.to_string() }
                }
                (Some(cho), Some(jung), None) => {
                    // Check if consonant can be jongseong
                    if JONGSEONG_LIST.contains(&ch) {
                        self.jongseong = Some(ch);
                        let sy = make_syllable(cho, jung, Some(ch));
                        HangulAction::Update { backspaces: 1, text: sy.to_string() }
                    } else {
                        // Consonant cannot be jongseong (like ㄸ, ㅃ, ㅉ)
                        self.reset();
                        self.choseong = Some(ch);
                        HangulAction::Update { backspaces: 0, text: ch.to_string() }
                    }
                }
                (Some(cho), Some(jung), Some(jong)) => {
                    // Check compound jongseong
                    if let Some(comb) = combine_jongseongs(jong, ch) {
                        self.jongseong = Some(comb);
                        let sy = make_syllable(cho, jung, Some(comb));
                        HangulAction::Update { backspaces: 1, text: sy.to_string() }
                    } else {
                        // Cannot combine, commit previous syllable and start new
                        self.reset();
                        self.choseong = Some(ch);
                        HangulAction::Update { backspaces: 0, text: ch.to_string() }
                    }
                }
            }
        } else if is_vowel(ch) {
            match (self.choseong, self.jungseong, self.jongseong) {
                (None, None, _) => {
                    HangulAction::Update { backspaces: 0, text: ch.to_string() }
                }
                (Some(cho), None, _) => {
                    // Combine consonant + vowel
                    self.jungseong = Some(ch);
                    let sy = make_syllable(cho, ch, None);
                    HangulAction::Update { backspaces: 1, text: sy.to_string() }
                }
                (Some(cho), Some(jung), None) => {
                    // Combine compound vowel
                    if let Some(comb) = combine_vowels(jung, ch) {
                        self.jungseong = Some(comb);
                        let sy = make_syllable(cho, comb, None);
                        HangulAction::Update { backspaces: 1, text: sy.to_string() }
                    } else {
                        self.reset();
                        HangulAction::Update { backspaces: 0, text: ch.to_string() }
                    }
                }
                (Some(cho), Some(jung), Some(jong)) => {
                    // Consonant migration: trailing consonant moves to new syllable
                    if let Some((first, second)) = decompose_jongseong(jong) {
                        // Previous syllable retains first half of compound jongseong
                        let sy1 = make_syllable(cho, jung, Some(first));
                        // New syllable starts with second half
                        let sy2 = make_syllable(second, ch, None);
                        self.choseong = Some(second);
                        self.jungseong = Some(ch);
                        self.jongseong = None;
                        HangulAction::Update { backspaces: 1, text: format!("{}{}", sy1, sy2) }
                    } else {
                        // Previous syllable drops jongseong completely
                        let sy1 = make_syllable(cho, jung, None);
                        // New syllable starts with the single jongseong consonant
                        let sy2 = make_syllable(jong, ch, None);
                        self.choseong = Some(jong);
                        self.jungseong = Some(ch);
                        self.jongseong = None;
                        HangulAction::Update { backspaces: 1, text: format!("{}{}", sy1, sy2) }
                    }
                }
                _ => {
                    self.reset();
                    HangulAction::Update { backspaces: 0, text: ch.to_string() }
                }
            }
        } else {
            self.reset();
            HangulAction::ResetAndType(ch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hangul_compose() {
        let mut composer = HangulComposer::new();

        // Test typing "강"
        // 1. feed 'ㄱ'
        if let HangulAction::Update { backspaces, text } = composer.feed('ㄱ') {
            assert_eq!(backspaces, 0);
            assert_eq!(text, "ㄱ");
        } else { panic!(); }

        // 2. feed 'ㅏ' -> "가"
        if let HangulAction::Update { backspaces, text } = composer.feed('ㅏ') {
            assert_eq!(backspaces, 1);
            assert_eq!(text, "가");
        } else { panic!(); }

        // 3. feed 'ㅇ' -> "강"
        if let HangulAction::Update { backspaces, text } = composer.feed('ㅇ') {
            assert_eq!(backspaces, 1);
            assert_eq!(text, "강");
        } else { panic!(); }

        // 4. feed 'ㅏ' -> consonant migration: "가아"
        if let HangulAction::Update { backspaces, text } = composer.feed('ㅏ') {
            assert_eq!(backspaces, 1);
            assert_eq!(text, "가아");
        } else { panic!(); }
    }
}
