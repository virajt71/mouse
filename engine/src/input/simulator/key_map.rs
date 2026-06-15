use evdev::Key;
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref KEY_MAP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("ctrl", Key::KEY_LEFTCTRL);
        m.insert("crtl", Key::KEY_LEFTCTRL);
        m.insert("control", Key::KEY_LEFTCTRL);
        m.insert("shift", Key::KEY_LEFTSHIFT);
        m.insert("alt", Key::KEY_LEFTALT);
        m.insert("opt", Key::KEY_LEFTALT);
        m.insert("option", Key::KEY_LEFTALT);
        m.insert("super", Key::KEY_LEFTMETA);
        m.insert("meta", Key::KEY_LEFTMETA);
        m.insert("cmd", Key::KEY_LEFTMETA);
        m.insert("command", Key::KEY_LEFTMETA);
        m.insert("win", Key::KEY_LEFTMETA);
        m.insert("windows", Key::KEY_LEFTMETA);
        m.insert("tab", Key::KEY_TAB);
        m.insert("space", Key::KEY_SPACE);
        m.insert("enter", Key::KEY_ENTER);
        m.insert("esc", Key::KEY_ESC);
        m.insert("backspace", Key::KEY_BACKSPACE);
        m.insert("delete", Key::KEY_DELETE);
        m.insert("left", Key::KEY_LEFT);
        m.insert("right", Key::KEY_RIGHT);
        m.insert("up", Key::KEY_UP);
        m.insert("down", Key::KEY_DOWN);
        m.insert("pageup", Key::KEY_PAGEUP);
        m.insert("pagedown", Key::KEY_PAGEDOWN);
        m.insert("home", Key::KEY_HOME);
        m.insert("end", Key::KEY_END);
        m.insert("volumeup", Key::KEY_VOLUMEUP);
        m.insert("volumedown", Key::KEY_VOLUMEDOWN);
        m.insert("mute", Key::KEY_MUTE);
        m.insert("playpause", Key::KEY_PLAYPAUSE);
        m.insert("nexttrack", Key::KEY_NEXTSONG);
        m.insert("prevtrack", Key::KEY_PREVIOUSSONG);

        // A-Z
        m.insert("a", Key::KEY_A);
        m.insert("b", Key::KEY_B);
        m.insert("c", Key::KEY_C);
        m.insert("d", Key::KEY_D);
        m.insert("e", Key::KEY_E);
        m.insert("f", Key::KEY_F);
        m.insert("g", Key::KEY_G);
        m.insert("h", Key::KEY_H);
        m.insert("i", Key::KEY_I);
        m.insert("j", Key::KEY_J);
        m.insert("k", Key::KEY_K);
        m.insert("l", Key::KEY_L);
        m.insert("m", Key::KEY_M);
        m.insert("n", Key::KEY_N);
        m.insert("o", Key::KEY_O);
        m.insert("p", Key::KEY_P);
        m.insert("q", Key::KEY_Q);
        m.insert("r", Key::KEY_R);
        m.insert("s", Key::KEY_S);
        m.insert("t", Key::KEY_T);
        m.insert("u", Key::KEY_U);
        m.insert("v", Key::KEY_V);
        m.insert("w", Key::KEY_W);
        m.insert("x", Key::KEY_X);
        m.insert("y", Key::KEY_Y);
        m.insert("z", Key::KEY_Z);

        // 0-9
        m.insert("0", Key::KEY_0);
        m.insert("1", Key::KEY_1);
        m.insert("2", Key::KEY_2);
        m.insert("3", Key::KEY_3);
        m.insert("4", Key::KEY_4);
        m.insert("5", Key::KEY_5);
        m.insert("6", Key::KEY_6);
        m.insert("7", Key::KEY_7);
        m.insert("8", Key::KEY_8);
        m.insert("9", Key::KEY_9);

        // F1-F12
        m.insert("f1", Key::KEY_F1);
        m.insert("f2", Key::KEY_F2);
        m.insert("f3", Key::KEY_F3);
        m.insert("f4", Key::KEY_F4);
        m.insert("f5", Key::KEY_F5);
        m.insert("f6", Key::KEY_F6);
        m.insert("f7", Key::KEY_F7);
        m.insert("f8", Key::KEY_F8);
        m.insert("f9", Key::KEY_F9);
        m.insert("f10", Key::KEY_F10);
        m.insert("f11", Key::KEY_F11);
        m.insert("f12", Key::KEY_F12);
        m.insert("equal", Key::KEY_EQUAL);
        m.insert("minus", Key::KEY_MINUS);
        m.insert("brightnessup", Key::KEY_BRIGHTNESSUP);
        m.insert("brightnessdown", Key::KEY_BRIGHTNESSDOWN);

        m
    };
}

pub fn get_key_by_layout(key: &str, layout: &str) -> Option<Key> {
    match layout {
        // QWERTZ layouts
        "QWERTZ (German)"
        | "German"
        | "Hungarian"
        | "Swiss German/French"
        | "Slovak"
        | "Bosnian"
        | "Croatian"
        | "Czech"
        | "Polish (214)"
        | "Serbian"
        | "Serbian (Latin)"
        | "Slovene/Slovenian" => match key {
            "y" => Some(Key::KEY_Z),
            "z" => Some(Key::KEY_Y),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // AZERTY layouts
        "AZERTY (French)" | "French" | "Belgian" => match key {
            "a" => Some(Key::KEY_Q),
            "q" => Some(Key::KEY_A),
            "z" => Some(Key::KEY_W),
            "w" => Some(Key::KEY_Z),
            "m" => Some(Key::KEY_SEMICOLON),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // Dvorak English layout
        "Dvorak English" => match key {
            "a" => Some(Key::KEY_A),
            "b" => Some(Key::KEY_N),
            "c" => Some(Key::KEY_I),
            "d" => Some(Key::KEY_H),
            "e" => Some(Key::KEY_D),
            "f" => Some(Key::KEY_Y),
            "g" => Some(Key::KEY_U),
            "h" => Some(Key::KEY_J),
            "i" => Some(Key::KEY_G),
            "j" => Some(Key::KEY_C),
            "k" => Some(Key::KEY_V),
            "l" => Some(Key::KEY_P),
            "m" => Some(Key::KEY_M),
            "n" => Some(Key::KEY_L),
            "o" => Some(Key::KEY_S),
            "p" => Some(Key::KEY_R),
            "q" => Some(Key::KEY_X),
            "r" => Some(Key::KEY_O),
            "s" => Some(Key::KEY_SEMICOLON),
            "t" => Some(Key::KEY_K),
            "u" => Some(Key::KEY_F),
            "v" => Some(Key::KEY_DOT),
            "w" => Some(Key::KEY_COMMA),
            "x" => Some(Key::KEY_B),
            "y" => Some(Key::KEY_T),
            "z" => Some(Key::KEY_SLASH),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // Colemak English layout
        "Colemak English" => match key {
            "a" => Some(Key::KEY_A),
            "b" => Some(Key::KEY_B),
            "c" => Some(Key::KEY_C),
            "d" => Some(Key::KEY_D),
            "e" => Some(Key::KEY_J),
            "f" => Some(Key::KEY_E),
            "g" => Some(Key::KEY_T),
            "h" => Some(Key::KEY_G),
            "i" => Some(Key::KEY_K),
            "j" => Some(Key::KEY_Y),
            "k" => Some(Key::KEY_N),
            "l" => Some(Key::KEY_U),
            "m" => Some(Key::KEY_M),
            "n" => Some(Key::KEY_H),
            "o" => Some(Key::KEY_L),
            "p" => Some(Key::KEY_R),
            "q" => Some(Key::KEY_Q),
            "r" => Some(Key::KEY_SEMICOLON),
            "s" => Some(Key::KEY_S),
            "t" => Some(Key::KEY_F),
            "u" => Some(Key::KEY_I),
            "v" => Some(Key::KEY_V),
            "w" => Some(Key::KEY_W),
            "x" => Some(Key::KEY_X),
            "y" => Some(Key::KEY_O),
            "z" => Some(Key::KEY_Z),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // French (BÉPO) layout
        "French (BÉPO)" => match key {
            "a" => Some(Key::KEY_A),
            "b" => Some(Key::KEY_Q),
            "c" => Some(Key::KEY_H),
            "d" => Some(Key::KEY_I),
            "e" => Some(Key::KEY_F),
            "f" => Some(Key::KEY_DOT),
            "g" => Some(Key::KEY_M),
            "h" => Some(Key::KEY_COMMA),
            "i" => Some(Key::KEY_D),
            "j" => Some(Key::KEY_P),
            "k" => Some(Key::KEY_V),
            "l" => Some(Key::KEY_O),
            "m" => Some(Key::KEY_APOSTROPHE),
            "n" => Some(Key::KEY_SEMICOLON),
            "o" => Some(Key::KEY_R),
            "p" => Some(Key::KEY_E),
            "q" => Some(Key::KEY_N),
            "r" => Some(Key::KEY_L),
            "s" => Some(Key::KEY_K),
            "t" => Some(Key::KEY_J),
            "u" => Some(Key::KEY_S),
            "v" => Some(Key::KEY_U),
            "w" => Some(Key::KEY_RIGHTBRACE),
            "x" => Some(Key::KEY_C),
            "y" => Some(Key::KEY_X),
            "z" => Some(Key::KEY_LEFTBRACE),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // Turkish F layout
        "Turkish F" => match key {
            "a" => Some(Key::KEY_F),
            "b" => Some(Key::KEY_COMMA),
            "c" => Some(Key::KEY_V),
            "d" => Some(Key::KEY_Y),
            "e" => Some(Key::KEY_D),
            "f" => Some(Key::KEY_Q),
            "g" => Some(Key::KEY_W),
            "h" => Some(Key::KEY_O),
            "i" => Some(Key::KEY_S),
            "j" => Some(Key::KEY_Z),
            "k" => Some(Key::KEY_J),
            "l" => Some(Key::KEY_L),
            "m" => Some(Key::KEY_K),
            "n" => Some(Key::KEY_I),
            "o" => Some(Key::KEY_T),
            "p" => Some(Key::KEY_P),
            "q" => Some(Key::KEY_LEFTBRACE),
            "r" => Some(Key::KEY_U),
            "s" => Some(Key::KEY_M),
            "t" => Some(Key::KEY_H),
            "u" => Some(Key::KEY_A),
            "v" => Some(Key::KEY_C),
            "w" => Some(Key::KEY_RIGHTBRACE),
            "y" => Some(Key::KEY_SEMICOLON),
            "z" => Some(Key::KEY_N),
            _ => (*KEY_MAP).get(key).cloned(),
        },

        // Default QWERTY fallback
        _ => (*KEY_MAP).get(key).cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_german_qwertz_mapping() {
        assert_eq!(get_key_by_layout("y", "QWERTZ (German)"), Some(Key::KEY_Z));
        assert_eq!(get_key_by_layout("z", "QWERTZ (German)"), Some(Key::KEY_Y));
        assert_eq!(get_key_by_layout("a", "QWERTZ (German)"), Some(Key::KEY_A));

        assert_eq!(get_key_by_layout("y", "German"), Some(Key::KEY_Z));
        assert_eq!(get_key_by_layout("z", "German"), Some(Key::KEY_Y));
        assert_eq!(get_key_by_layout("y", "Hungarian"), Some(Key::KEY_Z));
        assert_eq!(get_key_by_layout("z", "Hungarian"), Some(Key::KEY_Y));
    }

    #[test]
    fn test_french_azerty_mapping() {
        assert_eq!(get_key_by_layout("a", "AZERTY (French)"), Some(Key::KEY_Q));
        assert_eq!(get_key_by_layout("q", "AZERTY (French)"), Some(Key::KEY_A));
        assert_eq!(get_key_by_layout("z", "AZERTY (French)"), Some(Key::KEY_W));
        assert_eq!(get_key_by_layout("w", "AZERTY (French)"), Some(Key::KEY_Z));
        assert_eq!(
            get_key_by_layout("m", "AZERTY (French)"),
            Some(Key::KEY_SEMICOLON)
        );
        assert_eq!(get_key_by_layout("b", "AZERTY (French)"), Some(Key::KEY_B));

        assert_eq!(get_key_by_layout("a", "French"), Some(Key::KEY_Q));
        assert_eq!(get_key_by_layout("a", "Belgian"), Some(Key::KEY_Q));
    }

    #[test]
    fn test_dvorak_mapping() {
        assert_eq!(get_key_by_layout("b", "Dvorak English"), Some(Key::KEY_N));
        assert_eq!(get_key_by_layout("c", "Dvorak English"), Some(Key::KEY_I));
        assert_eq!(
            get_key_by_layout("z", "Dvorak English"),
            Some(Key::KEY_SLASH)
        );
    }

    #[test]
    fn test_colemak_mapping() {
        assert_eq!(get_key_by_layout("e", "Colemak English"), Some(Key::KEY_J));
        assert_eq!(get_key_by_layout("f", "Colemak English"), Some(Key::KEY_E));
        assert_eq!(get_key_by_layout("y", "Colemak English"), Some(Key::KEY_O));
    }

    #[test]
    fn test_bepo_mapping() {
        assert_eq!(get_key_by_layout("b", "French (BÉPO)"), Some(Key::KEY_Q));
        assert_eq!(
            get_key_by_layout("w", "French (BÉPO)"),
            Some(Key::KEY_RIGHTBRACE)
        );
        assert_eq!(
            get_key_by_layout("z", "French (BÉPO)"),
            Some(Key::KEY_LEFTBRACE)
        );
    }

    #[test]
    fn test_turkish_f_mapping() {
        assert_eq!(get_key_by_layout("a", "Turkish F"), Some(Key::KEY_F));
        assert_eq!(get_key_by_layout("u", "Turkish F"), Some(Key::KEY_A));
        assert_eq!(get_key_by_layout("z", "Turkish F"), Some(Key::KEY_N));
    }

    #[test]
    fn test_default_ansi_mapping() {
        assert_eq!(get_key_by_layout("y", "ANSI (US)"), Some(Key::KEY_Y));
        assert_eq!(get_key_by_layout("z", "ANSI (US)"), Some(Key::KEY_Z));
        assert_eq!(get_key_by_layout("m", "ANSI (US)"), Some(Key::KEY_M));
    }
}
