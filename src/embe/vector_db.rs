use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct VectorDB {
    client: Client,
    es_url: String,
    index_name: String,
}

impl VectorDB {
    pub fn new(es_url: &str, index_name: &str) -> Self {
        VectorDB {
            client: Client::new(),
            es_url: es_url.to_string(),
            index_name: index_name.to_string(),
        }
    }

    pub async fn store_vector(&self, id: &str, vector: &[f32]) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/{}/_doc/{}", self.es_url, self.index_name, id);
        let body = json!({
            "vector": vector,
        });

        self.client.put(&url)
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn query_similar_vectors(&self, query_vector: &[f32], top_k: usize) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        let url = format!("{}/{}/_search", self.es_url, self.index_name);
        let body = json!({
            "size": top_k,
            "query": {
                "script_score": {
                    "query": {"match_all": {}},
                    "script": {
                        "source": "cosineSimilarity(params.query_vector, 'vector') + 1.0",
                        "params": {"query_vector": query_vector}
                    }
                }
            }
        });

        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let hits = response["hits"]["hits"].as_array()
            .ok_or("无效的响应格式")?;

        let results = hits.iter()
            .map(|hit| {
                let id = hit["_id"].as_str().unwrap_or("").to_string();
                let score = hit["_score"].as_f64().unwrap_or(0.0) as f32;
                (id, score)
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_vector_db() {
        let es_url = "http://localhost:9200";  // 请替换为你的Elasticsearch URL
        let index_name = "test_vector_index";
        let db = VectorDB::new(es_url, index_name);

        // 存储向量
        let id = "test_vector_1";
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        db.store_vector(id, &vector).await.unwrap();

        // 查询相似向量
        let query_vector = vec![0.15, 0.25, 0.35, 0.45, 0.55];
        let results = db.query_similar_vectors(&query_vector, 5).await.unwrap();

        assert!(!results.is_empty(), "查询结果不应为空");
        println!("查询结果: {:?}", results);
    }
}
