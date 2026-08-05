// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Default)]
pub struct ContextViewEngine {
    visible_items: usize,
    locked: bool,
}

impl ContextViewEngine {
    pub fn sync(&mut self, visible_items: usize, locked: bool) {
        self.visible_items = visible_items;
        self.locked = locked;
    }

    pub fn summary(&self) -> String {
        format!(
            "{} items · {}",
            self.visible_items,
            if self.locked { "locked" } else { "open" }
        )
    }
}
