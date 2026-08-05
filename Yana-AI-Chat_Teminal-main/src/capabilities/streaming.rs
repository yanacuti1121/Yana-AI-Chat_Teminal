// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Default)] pub struct StreamingEngine { active: bool, chunks: usize }
impl StreamingEngine { pub fn start(&mut self) { self.active = true; self.chunks = 0; } pub fn push_chunk(&mut self) { if self.active { self.chunks += 1; } } pub fn finish(&mut self) { self.active = false; } pub fn active(&self) -> bool { self.active } pub fn chunks(&self) -> usize { self.chunks } }
