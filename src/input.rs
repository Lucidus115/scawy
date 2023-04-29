use std::collections::{HashMap, HashSet};

use game_loop::winit::event::{ElementState, VirtualKeyCode};

pub type KeyCode = VirtualKeyCode;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct KeyState {
    just_changed: bool,
    current: ElementState,
}

#[derive(Default)]
pub struct KeyboardInput {
    state: HashMap<KeyCode, KeyState>,
}

impl KeyboardInput {
    pub fn pressed(&self, key: KeyCode) -> bool {
        let Some(state) = self.state.get(&key) else {
            return false;
        };
        state.current == ElementState::Pressed && state.just_changed
    }

    pub fn released(&self, key: KeyCode) -> bool {
        let Some(state) = self.state.get(&key) else {
            return false;
        };
        state.current == ElementState::Released
    }

    pub fn held(&self, key: KeyCode) -> bool {
        let Some(state) = self.state.get(&key) else {
            return false;
        };
        state.current == ElementState::Pressed
    }

    pub(crate) fn capture_keys(&mut self, keys: &mut Vec<game_loop::winit::event::KeyboardInput>) {
        // reset just changed bool for keys
        self.state
            .values_mut()
            .filter(|val| val.just_changed)
            .for_each(|val| val.just_changed = false);

        // capture input and then remove keys from list
        let mut keys_checked = HashSet::new();
        keys.reverse();
        keys.retain(|key| {
            if keys_checked.contains(key) {
                return false;
            }

            let Some(code) = key.virtual_keycode else {
                return false;
            };

            // check if prev state is the same in case key event fired twice
            if let Some(state) = self.state.get(&code) {

                // state never changed so no need to do anything
                if state.current == key.state {
                    return false;
                }
            }

            self.state.insert(
                code,
                KeyState {
                    just_changed: true,
                    current: key.state,
                },
            );

            keys_checked.insert(*key);
            false
        });
    }
}
