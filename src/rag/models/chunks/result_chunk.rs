use qdrant_client::qdrant::ScoredPoint;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ResultChunk {
    pub id: String,
    pub doc_id: String,
    pub doc_seq_num: i32,
    pub content: String,
    pub additional_data: Value,
    pub doc_summary: String,
    pub score: f32,
}

impl From<ScoredPoint> for ResultChunk {
    fn from(value: ScoredPoint) -> Self {
        let id: String = match value.id {
            Some(d) => format!("{:?}", d),
            None => "Unknown".into(),
        };

        let doc_id = match value.payload.get("doc_id") {
            Some(d) => d.as_str().map_or("Unknown", |v| v),
            None => "Unknown",
        };
        let doc_id = doc_id.to_string();

        let doc_seq_num = match value.payload.get("doc_seq_num") {
            Some(d) => d.as_integer().unwrap_or(-1) as i32,
            None => -1,
        };

        let content: String = match value.payload.get("content") {
            Some(d) => d.as_str().map_or("".into(), |v| v.into()),
            None => "".into(),
        };

        let additional_data = match value.payload.get("additional_data") {
            Some(d) => d.to_owned(),
            None => Value::Null.into(),
        };

        let doc_summary: String = match value.payload.get("doc_summary") {
            Some(d) => d.as_str().map_or("".into(), |v| v.into()),
            None => "".into(),
        };      

        Self {
            id,
            doc_id,
            doc_seq_num,
            doc_summary,
            content,
            additional_data: additional_data.into(),
            score: value.score,
        }
    }
}

impl ResultChunk {
    pub fn to_prompt_chunk(&self) -> String {
        let link = match &self.additional_data {
            Value::Array(vec) => vec
                .get(1)
                .unwrap_or( &Value::String("None".into()))
                .to_string(),
            _ => "None".to_string()
        };

        
        let link = if link.eq("None") {
            "".to_string()
        } else {
            format!("\tPARENT DOCUMENT ADDITIONAL DATA: {}", link)
        };
        let doc_summary = &self.doc_summary;
        
        
        format!(
            "CHUNK:\n\tPARENT DOCUMENT DESCRIPTION:\n{}\n{}\n\n\tCHUNK CONTENTS: {}", 
            doc_summary,
            link,
            self.content
        )
    }
}