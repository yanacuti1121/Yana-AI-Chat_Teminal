// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest { pub id: u64, pub title: String, pub detail: String }
#[derive(Debug, Default)]
pub struct ApprovalEngine { next_id: u64, pending: Vec<ApprovalRequest> }
impl ApprovalEngine {
    pub fn request(&mut self, title: impl Into<String>, detail: impl Into<String>) -> u64 { let id = if self.next_id == 0 { 1 } else { self.next_id }; self.next_id = id + 1; self.pending.push(ApprovalRequest { id, title: title.into(), detail: detail.into() }); id }
    pub fn resolve(&mut self, id: u64) -> Option<ApprovalRequest> { let index = self.pending.iter().position(|request| request.id == id)?; Some(self.pending.remove(index)) }
    pub fn latest(&self) -> Option<&ApprovalRequest> { self.pending.last() }
    pub fn pending(&self) -> &[ApprovalRequest] { &self.pending }
}
