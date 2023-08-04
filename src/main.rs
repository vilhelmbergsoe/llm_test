use std::io::Write;
use llm::Model;

fn main() {
    let llama = llm::load::<llm::models::Llama>(
        // path to GGML file
        std::path::Path::new("../../../Downloads/Wizard-Vicuna-7B-Uncensored.ggmlv3.q4_0.bin"),
        llm::TokenizerSource::Embedded,
        // llm::ModelParameters
        Default::default(),
        // load progress callback
        llm::load_progress_callback_stdout
    )
    .unwrap_or_else(|err| panic!("Failed to load model: {err}"));

    // use the model to generate text from a prompt
    let mut session = llama.start_session(Default::default());
    let res = session.infer::<std::convert::Infallible>(
        // model to use for text generation
        &llama,
        // randomness provider
        &mut rand::thread_rng(),
        // the prompt to use for text generation, as well as other
        // inference parameters
        &llm::InferenceRequest {
            prompt: llm::Prompt::Text("Rust is a cool programming language because"),
            maximum_token_count: None,
            parameters: &llm::InferenceParameters {..Default::default()},
            play_back_previous_tokens: false,
        },
        // llm::OutputRequest
        &mut Default::default(),
        // output callback
        |r| match r {
            llm::InferenceResponse::PromptToken(t) | llm::InferenceResponse::InferredToken(t) => {
                print!("{t}");
                std::io::stdout().flush().unwrap();

                Ok(llm::InferenceFeedback::Continue)
            }
            _ => Ok(llm::InferenceFeedback::Continue),
        },
    );

    match res {
        Ok(result) => println!("\n\nInference stats:\n{result}"),
        Err(err) => println!("\n{err}"),
    }
}
