use reqwest;
use serde_json::{json, Value};

pub async fn get_baidu_embedding(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let api_key = "你的百度API密钥";  // 请替换为你的实际API密钥
    let secret_key = "你的百度密钥";  // 请替换为你的实际密钥

    // 获取访问令牌
    let token_url = format!("https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}", api_key, secret_key);
    let token_response = client.get(&token_url).send().await?;
    let token_json: Value = token_response.json().await?;
    let access_token = token_json["access_token"].as_str().unwrap();

    // 调用文本嵌入API
    let embedding_url = format!("https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/embeddings/embedding-v1?access_token={}", access_token);
    let response = client.post(&embedding_url)
        .json(&json!({
            "input": text,
        }))
        .send()
        .await?;

    let result: Value = response.json().await?;
    
    // 解析嵌入向量
    let embedding = result["data"][0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();

    Ok(embedding)
}

// 使用示例
// let embedding = get_baidu_embedding("这是一段测试文本").await?;
// println!("嵌入向量: {:?}", embedding);

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_baidu_embedding() {
        let result = get_baidu_embedding("这是一段测试文本").await;
        assert!(result.is_ok(), "获取百度嵌入向量失败");

        let embedding = result.unwrap();
        assert!(!embedding.is_empty(), "嵌入向量不应为空");
        assert_eq!(embedding.len(), 384, "嵌入向量的维度应为384");

        // 检查向量的值是否在合理范围内
        for &value in &embedding {
            assert!((-1.0..=1.0).contains(&value), "嵌入向量的值应在 -1 到 1 之间");
        }

        println!("嵌入向量的前5个元素: {:?}", &embedding[..5]);
    }
}

