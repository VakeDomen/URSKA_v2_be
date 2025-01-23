use ollama_rs::generation::{completion::request::GenerationRequest, parameters::{FormatType, JsonStructure}};


#[derive(Debug, Clone)]
pub struct StructuredQuestion {
    system_prompt: String,
    question: String,
    context: Vec<String>,
    model: String,
    format: JsonStructure,
}

impl From<(String, JsonStructure)> for StructuredQuestion {
    fn from(values: (String, JsonStructure)) -> Self {
        Self {
            system_prompt: "You are a helpful assistant. Answer users question based on provided context.".to_owned(),
            question: values.0,
            context: vec![],
            model: "phi4".to_owned(),
            format: values.1,
        }
    }
}

impl From<(&str, JsonStructure)> for StructuredQuestion {
    fn from(values: (&str, JsonStructure)) -> Self {
        Self {
            system_prompt: "You are a helpful assistant. Answer users question based on provided context.".to_owned(),
            question: values.0.to_owned(),
            context: vec![],
            model: "phi4".to_owned(),
            format: values.1,
        }
    }
}

impl Into<GenerationRequest> for StructuredQuestion {
    fn into(self) -> GenerationRequest {
        let context = if self.context.is_empty() {
            "".to_string()
        } else {
            self.context.join("\n")
        };
        
        let final_prompt = format!(
            "{}\n{}\n{}", 
            self.system_prompt,
            self.question,
            context
        );

        
        GenerationRequest::new(self.model, final_prompt)
            .format(FormatType::StructuredJson(self.format))
    }
}

impl StructuredQuestion {
    pub fn set_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn set_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
    
    pub fn set_question(mut self, question: &str) -> Self {
        self.question = question.to_string();
        self
    }

    pub fn set_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }
}
