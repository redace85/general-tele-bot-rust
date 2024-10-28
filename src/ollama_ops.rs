use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct GenerateRes {
    model: String,
    response: String,
    done: bool,
    done_reason: String,
    total_duration: u64,
}

pub async fn model_generate(
    ollama_server: &str,
    model: &str,
    prompt: String,
) -> Result<String, String> {
    let req_data = json!( {
            "model": model,
            "prompt": prompt,
            "stream": false
          }
    );

    let client = reqwest::Client::new();

    let req_result = client
        .post(format!("{ollama_server}/api/generate"))
        .json(&req_data)
        .send()
        .await;

    let res = match req_result {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    if !res.status().is_success() {
        return Err(format!("request failed with status code:{}", res.status()));
    };

    let gen_res = match res.json::<GenerateRes>().await {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    Ok(gen_res.response)
}

// for github pipline to pass
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_ollama_req() {
//         let prompt = "道爷说要相信科学".into();
//         let ret_text = model_generate("http://localhost:11434", "qwen2.5:7b", prompt ).await.unwrap();
//         println!("output :{ret_text}",);
//     }
// }
