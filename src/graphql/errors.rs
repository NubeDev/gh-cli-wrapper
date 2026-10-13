//! The `errors` array a GraphQL 200 can carry.

/// The `errors` array a GraphQL 200 can carry, flattened to one message.
///
/// `None` when the response has none. Handles both shapes `--paginate` produces: a
/// single object, and an array of page objects.
pub fn graphql_errors(value: &serde_json::Value) -> Option<String> {
    let pages: Vec<&serde_json::Value> = match value.as_array() {
        Some(pages) => pages.iter().collect(),
        None => vec![value],
    };

    let messages: Vec<String> = pages
        .iter()
        .filter_map(|page| page.get("errors").and_then(|e| e.as_array()))
        .flatten()
        .map(|error| {
            error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unspecified GraphQL error")
                .to_string()
        })
        .collect();

    (!messages.is_empty()).then(|| messages.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A GraphQL 200 carrying `errors` alongside a partial `data` is a failure. Reading
    /// it as success is how a partial answer gets reported as a complete one.
    #[test]
    fn errors_beside_data_are_still_errors() {
        let body = serde_json::json!({
            "data": { "repository": null },
            "errors": [{ "message": "Could not resolve to a Repository with the name 'a/b'." }]
        });
        let message = graphql_errors(&body).unwrap();
        assert!(message.contains("Could not resolve"), "{message}");
    }

    #[test]
    fn a_clean_response_has_no_errors() {
        assert!(graphql_errors(&serde_json::json!({"data": {"x": 1}})).is_none());
    }

    /// `--paginate` yields an array of page objects; an error on page two must not be
    /// lost because page one was clean.
    #[test]
    fn an_error_on_a_later_page_is_found() {
        let body = serde_json::json!([
            { "data": { "x": 1 } },
            { "data": {}, "errors": [{ "message": "rate limited" }] }
        ]);
        assert_eq!(graphql_errors(&body).unwrap(), "rate limited");
    }
}
