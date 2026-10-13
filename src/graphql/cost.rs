//! What a GraphQL query cost, **asked in the query** — `rateLimit { cost }`.
//!
//! A `gh` subcommand such as `gh repo list` spends GraphQL too but never says how much, so
//! its cost is unknown rather than guessed. A query this crate sends says.

/// `data.rateLimit.cost`, summed over every page `--paginate` printed. `None` when no page
/// carried one (the query did not ask, or GitHub did not answer).
pub fn graphql_cost(stdout: &str) -> Option<u64> {
    let mut total = None;
    for page in serde_json::Deserializer::from_str(stdout).into_iter::<serde_json::Value>() {
        let Ok(page) = page else { break };
        let pages = match page {
            serde_json::Value::Array(pages) => pages,
            one => vec![one],
        };
        for page in pages {
            if let Some(cost) = page
                .pointer("/data/rateLimit/cost")
                .and_then(|c| c.as_u64())
            {
                total = Some(total.unwrap_or(0) + cost);
            }
        }
    }
    total
}

/// Ask GitHub what a query cost, **in the query**: `rateLimit { cost }` beside its first
/// selection. A mutation is left alone — `rateLimit` is a field of `Query` only.
pub fn with_cost(query: &str) -> String {
    let head = query.trim_start();
    if head.starts_with("mutation")
        || head.starts_with("subscription")
        || query.contains("rateLimit")
    {
        return query.to_string();
    }
    match query.find('{') {
        Some(at) => format!(
            "{} rateLimit {{ cost }} {}",
            &query[..=at],
            &query[at + 1..]
        ),
        None => query.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_asks_its_cost_and_a_mutation_does_not() {
        assert_eq!(
            with_cost("query { viewer { login } }"),
            "query { rateLimit { cost }  viewer { login } }"
        );
        let mutation = "mutation { addStar(input: {}) { clientMutationId } }";
        assert_eq!(with_cost(mutation), mutation);
    }

    #[test]
    fn the_cost_is_summed_over_pages() {
        let pages = r#"{"data":{"rateLimit":{"cost":3}}}
{"data":{"rateLimit":{"cost":2}}}"#;
        assert_eq!(graphql_cost(pages), Some(5));
        assert_eq!(graphql_cost(r#"{"data":{}}"#), None);
    }
}
