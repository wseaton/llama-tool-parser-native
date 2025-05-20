#[cfg(test)]
mod tests {
    use backend::{Value, parse_python};

    #[test]
    fn test_basic() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let source = r#"[
    get_weather_forecast(location="Tokyo", days=7),search_hotels(location="Shinjuku", check_in_date="2024-05-20", check_out_date="2024-05-27", budget_max_per_night=50.0, guest_count=2),
    get_attractions(location="Tokyo", count=3, category="all"),
    convert_currency(amount=1000, from_currency="USD", to_currency="JPY")
    ]"#;

        match parse_python(source) {
            Ok(function_calls) => {
                tracing::info!("Parsed function calls: {:?}", function_calls);

                assert_eq!(function_calls.len(), 4);
                assert_eq!(function_calls[0].name, "get_weather_forecast");
                assert_eq!(function_calls[1].name, "search_hotels");
                assert_eq!(function_calls[2].name, "get_attractions");
                assert_eq!(function_calls[3].name, "convert_currency");
                assert_eq!(function_calls[0].kwargs.len(), 2);
                assert_eq!(function_calls[1].kwargs.len(), 5);
                assert_eq!(function_calls[2].kwargs.len(), 3);
                assert_eq!(function_calls[3].kwargs.len(), 3);
                assert_eq!(
                    function_calls[0].kwargs.get("location"),
                    Some(&Value::String("Tokyo".to_string()))
                );
                assert_eq!(
                    function_calls[1].kwargs.get("location"),
                    Some(&Value::String("Shinjuku".to_string()))
                );
                assert_eq!(
                    function_calls[2].kwargs.get("location"),
                    Some(&Value::String("Tokyo".to_string()))
                );
            }
            Err((msg, span)) => {
                let error_message =
                    format!("Error at position {}-{}: {}", span.start, span.end, msg);
                tracing::error!("{}", error_message);
            }
        }
    }
}

fn main() {
    // This is just a placeholder main function to make the code compile.
    // The actual functionality is tested in the tests module.
    println!("Run tests with `cargo test`.");
}
