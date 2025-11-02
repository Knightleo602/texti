use crate::action::Action;
use crate::component::AppComponent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Deserializer};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

type KeyEventMap = HashMap<KeyEvent, Action>;
type ScreenMap = HashMap<AppComponent, KeyEventMap>;

#[derive(Clone, Default, Debug)]
pub struct Keybindings {
    map: ScreenMap,
}

impl Keybindings {
    pub fn with(map: ScreenMap) -> Self {
        Self { map }
    }
    pub fn get_action(&self, app_component: &AppComponent, key: KeyEvent) -> Option<Action> {
        self.map
            .get(app_component)
            .and_then(|map| map.get(&key))
            .cloned()
    }
    pub fn get_all_keybinds(
        &self,
        app_component: AppComponent,
    ) -> Option<Iter<'_, KeyEvent, Action>> {
        self.map.get(&app_component).map(|map| map.iter())
    }
    pub fn get_key_event_of_action(
        &self,
        app_component: AppComponent,
        action: Action,
    ) -> Option<KeyEvent> {
        let component_map = self.map.get(&app_component)?;
        for (ke, a) in component_map.iter() {
            if a == &action {
                return Some(*ke);
            }
        }
        None
    }
}

impl<'de> Deserialize<'de> for Keybindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map =
            <HashMap<AppComponent, HashMap<String, Action>>>::deserialize(deserializer)?;
        let keybindings: ScreenMap = parsed_map
            .into_iter()
            .map(|(comp, key_event_map)| {
                let converted: KeyEventMap = key_event_map
                    .into_iter()
                    .map(|(key, action)| (parse_key_event(&key).unwrap(), action))
                    .collect();
                (comp, converted)
            })
            .collect();
        Ok(Keybindings::with(keybindings))
    }
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

impl Deref for Keybindings {
    type Target = HashMap<AppComponent, KeyEventMap>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Keybindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

pub fn stringify_key_event(event: &KeyEvent) -> String {
    let key_string = event.code.to_string();
    let mut string_key = String::new();
    for modifier in event.modifiers {
        string_key.push_str(modifier.to_string().as_str());
        string_key.push('+')
    }
    string_key.push_str(&key_string);
    string_key
}
