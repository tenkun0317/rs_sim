use crate::block::Block;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct EditChange {
    pub x: i32,
    pub y: i32,
    pub old_block: Block,
    pub new_block: Block,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EditAction {
    pub changes: Vec<EditChange>,
}

pub struct History {
    pub undo_stack: Vec<EditAction>,
    pub redo_stack: Vec<EditAction>,
    current_action: Option<EditAction>,
}

impl History {
    pub fn new() -> Self {
        History {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_action: None,
        }
    }

    pub fn begin_action(&mut self) {
        self.current_action = Some(EditAction { changes: Vec::new() });
    }

    pub fn record(&mut self, x: i32, y: i32, old_block: Block, new_block: Block) {
        if let Some(ref mut action) = self.current_action {
            action.changes.push(EditChange { x, y, old_block, new_block });
        } else {
            let action = EditAction {
                changes: vec![EditChange { x, y, old_block, new_block }],
            };
            self.undo_stack.push(action);
            self.redo_stack.clear();
        }
    }

    pub fn end_action(&mut self) {
        if let Some(action) = self.current_action.take() {
            if !action.changes.is_empty() {
                self.undo_stack.push(action);
                self.redo_stack.clear();
            }
        }
    }

    pub fn undo(&mut self) -> Option<EditAction> {
        let action = self.undo_stack.pop()?;
        self.redo_stack.push(action.clone());
        Some(action)
    }

    pub fn redo(&mut self) -> Option<EditAction> {
        let action = self.redo_stack.pop()?;
        self.undo_stack.push(action.clone());
        Some(action)
    }

    pub fn stacks(&self) -> (&[EditAction], &[EditAction]) {
        (&self.undo_stack, &self.redo_stack)
    }
}
