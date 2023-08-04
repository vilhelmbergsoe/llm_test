use llm::{InferenceSession, Model};
use std::collections::VecDeque;
use std::fmt;
use std::io::Write;

struct Correspondence {
    user: String,
    assistant: String,
}

impl Correspondence {
    fn to_string(&self) -> String {
        format!("user:\n{}\nassistant:\n{}", self.user, self.assistant)
    }
}

struct Inference {
    system_prompt: String,
    context: String,
    correspondence_history: VecDeque<Correspondence>,
    max_context_length_words: usize,
}

impl fmt::Display for Inference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "System Prompt:\n{}", self.system_prompt)?;
        writeln!(f, "Context:\n{}", self.context)?;
        writeln!(f, "Correspondence History:")?;
        for correspondence in &self.correspondence_history {
            writeln!(f, "{}", correspondence.to_string())?;
        }
        Ok(())
    }
}

impl Inference {
    fn infer(
        &self,
        model: &llm::models::Llama,
        session: &mut InferenceSession, // model_params: ModelParams,
        user_input: &str,
    ) -> Correspondence {
        let mut input_text = format!(
            "System Prompt:\n{} \n\nContext:\n{} \n\nUser Input:\n{}",
            self.system_prompt, self.context, user_input
        );

        // reverse the correspondence_history to start checking from the most recent
        let mut history = self.correspondence_history.iter().rev();

        // buffer for reversed history
        let mut history_buffer = String::new();

        // append previous correspondences until reaching the max_context_length_words
        while input_text.split_whitespace().count() + history_buffer.split_whitespace().count()
            < self.max_context_length_words
        {
            match history.next() {
                Some(correspondence) => {
                    let correspondence_text = correspondence.to_string();
                    // check if adding this correspondence would exceed the limit
                    if input_text.split_whitespace().count()
                        + history_buffer.split_whitespace().count()
                        + correspondence_text.split_whitespace().count()
                        <= self.max_context_length_words
                    {
                        history_buffer = format!("{}\n\n{}", correspondence_text, history_buffer);
                    } else {
                        // if adding this correspondence exceeds the limit, stop the loop
                        break;
                    }
                }
                None => break,
            }
        }

        // prepend the history buffer to the input text
        input_text = format!(
            "Correspondence History:\n{} \n\n{}",
            history_buffer, input_text
        );

        // model inference here
        // let assistant_answer = model.infer(&model_params, &input_text);
        // let assistant_answer = "Okay!".to_string();
        let assistant_answer = {
            let mut response: String = "".to_string();
            let _ = session.infer::<std::convert::Infallible>(
                // model to use for text generation
                model,
                // randomness provider
                &mut rand::thread_rng(),
                // the prompt to use for text generation, as well as other
                // inference parameters
                &llm::InferenceRequest {
                    prompt: llm::Prompt::Text(input_text.as_str()),
                    maximum_token_count: None,
                    parameters: &llm::InferenceParameters {
                        ..Default::default()
                    },
                    play_back_previous_tokens: false,
                },
                // llm::OutputRequest
                &mut Default::default(),
                // output callback
                |r| match r {
                    llm::InferenceResponse::PromptToken(t)
                    | llm::InferenceResponse::InferredToken(t) => {
                        print!("{t}");
                        std::io::stdout().flush().unwrap();
                        response.push_str(&t);

                        Ok(llm::InferenceFeedback::Continue)
                    }
                    _ => Ok(llm::InferenceFeedback::Continue),
                },
            );

            response
        };

        // returns correspondence
        Correspondence {
            user: user_input.to_string(),
            assistant: assistant_answer,
        }
    }
}

fn main() {
    let mut infermodelthing = Inference {
        system_prompt: "You're a gorgeous AI assistant that is helpful.".to_string(),
        context: "This is some context".to_string(),
        correspondence_history: VecDeque::new(),
        max_context_length_words: 500,
    };

    infermodelthing
        .correspondence_history
        .push_back(Correspondence {
            user: "Hey there!".to_string(),
            assistant: "Hi! What can I help you with?".to_string(),
        });

    let llama = llm::load::<llm::models::Llama>(
        // path to GGML file
        std::path::Path::new("./models/Wizard-Vicuna-7B-Uncensored.ggmlv3.q4_0.bin"),
        llm::TokenizerSource::Embedded,
        // llm::ModelParameters
        Default::default(),
        // load progress callback
        llm::load_progress_callback_stdout,
    )
    .unwrap_or_else(|err| panic!("Failed to load model: {err}"));

    // use the model to generate text from a prompt
    let mut session = llama.start_session(Default::default());

    let res = infermodelthing.infer(&llama, &mut session, "What is 1 + 1?");
    infermodelthing.correspondence_history.push_back(res);

    // Display the state of the Inference instance
    println!("{}", infermodelthing);
}
