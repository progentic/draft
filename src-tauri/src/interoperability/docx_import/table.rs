use serde_json::{Value, json};

use super::DocxImportError;

#[derive(Default)]
pub(super) struct TableBuilder {
    current_cell: Option<Vec<Value>>,
    current_row: Option<Vec<Vec<Value>>>,
    rows: Vec<Vec<Vec<Value>>>,
}

impl TableBuilder {
    pub(super) fn start_row(&mut self) -> Result<(), DocxImportError> {
        if self.current_row.is_some() || self.current_cell.is_some() {
            return Err(DocxImportError::malformed());
        }
        self.current_row = Some(Vec::new());
        Ok(())
    }

    pub(super) fn start_cell(&mut self) -> Result<(), DocxImportError> {
        if self.current_row.is_none() || self.current_cell.is_some() {
            return Err(DocxImportError::malformed());
        }
        self.current_cell = Some(Vec::new());
        Ok(())
    }

    pub(super) fn push_blocks(&mut self, blocks: Vec<Value>) -> Result<(), DocxImportError> {
        self.current_cell
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .extend(blocks);
        Ok(())
    }

    pub(super) fn finish_cell(&mut self) -> Result<(), DocxImportError> {
        let cell = self
            .current_cell
            .take()
            .ok_or_else(DocxImportError::malformed)?;
        self.current_row
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .push(cell);
        Ok(())
    }

    pub(super) fn finish_row(&mut self) -> Result<(), DocxImportError> {
        if self.current_cell.is_some() {
            return Err(DocxImportError::malformed());
        }
        let row = self
            .current_row
            .take()
            .ok_or_else(DocxImportError::malformed)?;
        self.rows.push(row);
        Ok(())
    }

    pub(super) fn finish(self) -> Result<Vec<Value>, DocxImportError> {
        if self.current_cell.is_some() || self.current_row.is_some() {
            return Err(DocxImportError::malformed());
        }
        Ok(self.rows.into_iter().map(row_block).collect())
    }
}

fn row_block(row: Vec<Vec<Value>>) -> Value {
    let mut content = Vec::new();
    for (index, cell) in row.into_iter().enumerate() {
        if index > 0 {
            content.push(json!({ "type": "text", "text": " | " }));
        }
        content.extend(flatten_cell(cell));
    }
    if content.is_empty() {
        json!({ "type": "paragraph" })
    } else {
        json!({ "type": "paragraph", "content": content })
    }
}

fn flatten_cell(blocks: Vec<Value>) -> Vec<Value> {
    let mut content = Vec::new();
    for (index, block) in blocks.into_iter().enumerate() {
        if index > 0 {
            content.push(json!({ "type": "hardBreak" }));
        }
        append_block_content(&mut content, block);
    }
    content
}

fn append_block_content(content: &mut Vec<Value>, block: Value) {
    if block.get("type").and_then(Value::as_str) == Some("pageBreak") {
        content.push(json!({ "type": "hardBreak" }));
        return;
    }
    if let Some(nodes) = block.get("content").and_then(Value::as_array) {
        content.extend(nodes.iter().cloned());
    }
}
