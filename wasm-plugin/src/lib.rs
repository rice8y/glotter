use fasttext_pure_rs::FastText;
use std::io::Cursor;
use std::sync::OnceLock;
use wasm_minimal_protocol::*;

initiate_protocol!();

static MODEL: OnceLock<Result<FastText, String>> = OnceLock::new();

const MODEL_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/model/lid.176.ftz"
));

fn model() -> Result<&'static FastText, String> {
    MODEL
        .get_or_init(|| {
            FastText::load_from_reader(Cursor::new(MODEL_BYTES))
                .map_err(|error| format!("failed to load lid.176.ftz: {error}"))
        })
        .as_ref()
        .map_err(|error| error.clone())
}

fn parse_usize(bytes: &[u8], name: &str) -> Result<usize, String> {
    std::str::from_utf8(bytes)
        .map_err(|error| format!("invalid UTF-8 in {name}: {error}"))?
        .parse::<usize>()
        .map_err(|error| format!("invalid {name}: {error}"))
}

fn parse_f32(bytes: &[u8], name: &str) -> Result<f32, String> {
    std::str::from_utf8(bytes)
        .map_err(|error| format!("invalid UTF-8 in {name}: {error}"))?
        .parse::<f32>()
        .map_err(|error| format!("invalid {name}: {error}"))
}

fn normalize_label(label: &str) -> &str {
    label.strip_prefix("__label__").unwrap_or(label)
}

fn escape_json_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

fn predictions_to_json(
    predictions: impl IntoIterator<Item = fasttext_pure_rs::Prediction>,
) -> Vec<u8> {
    let mut json = String::from("[");

    for (index, prediction) in predictions.into_iter().enumerate() {
        if index > 0 {
            json.push(',');
        }

        let label = escape_json_string(prediction.label.as_str());
        let lang = escape_json_string(normalize_label(prediction.label.as_str()));

        json.push_str(&format!(
            "{{\"lang\":\"{lang}\",\"label\":\"{label}\",\"probability\":{}}}",
            prediction.probability
        ));
    }

    json.push(']');
    json.into_bytes()
}

#[wasm_func]
pub fn detect(text: &[u8], k: &[u8], threshold: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(text)
        .map_err(|error| format!("invalid UTF-8 text: {error}"))?;
    let k = parse_usize(k, "k")?;
    let threshold = parse_f32(threshold, "threshold")?;

    if text.trim().is_empty() {
        return Ok(b"[]".to_vec());
    }

    let predictions = model()?
        .predict(text, k, threshold)
        .map_err(|error| format!("prediction failed: {error}"))?;

    Ok(predictions_to_json(predictions))
}