use crate::rag::{comm::{question::Question, structured_qustion::StructuredQuestion, OllamaClient}, models::{chunks::ResultChunk, SearchResult}};
use ollama_rs::{error::OllamaError, generation::{completion::GenerationResponseStream, parameters::JsonStructure}};
use schemars::{schema_for, JsonSchema};



pub async fn recursive_prompt(prompt: String, chunks: Vec<ResultChunk>, ollama: &OllamaClient) -> Result<SearchResult, OllamaError> {
    let llm_prompt = construct_prompt(prompt, &chunks);
    let stream: GenerationResponseStream = ollama.generate_stream(llm_prompt).await?;
    Ok(SearchResult {
        chunks,
        stream,
    })
} 

#[derive(Debug, JsonSchema)]
pub struct TestFormat {
    resp: String,
    questions: Vec<String>,
}


fn construct_prompt(prompt: String, chunks: &Vec<ResultChunk>) -> StructuredQuestion {
    let system_message = "You are an assistant who is helping students find information \
        about University of Primorska. Your name is Urška. Given a \
        question, help navigate through the files and the information. You are allowed to read \
        some of the documents. Please answer in markdown format. When applicable add links. The uni \
        website is at https://www.famnit.upr.si ".to_string();


    let context: Vec<String> = chunks
        .iter()
        .map(|c| c.to_prompt_chunk())
        .collect();

    let question = format!(
        "{}\nQuestion:\n{}\n", 
        context.join("\n"),
        prompt
    );


    println!("{question}");

    StructuredQuestion::from((question, JsonStructure::new::<TestFormat>())).set_system_prompt(&system_message)
}