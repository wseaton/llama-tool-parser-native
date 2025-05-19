use backend::parse_python;

fn main() {
    let source = r#"[
get_weather_forecast(location="Tokyo", days=7),search_hotels(location="Shinjuku", check_in_date="2024-05-20", check_out_date="2024-05-27", budget_max_per_night=50.0, guest_count=2),
get_attractions(location="Tokyo", count=3, category="all"),
convert_currency(amount=1000, from_currency="USD", to_currency="JPY")
]"#;

    match parse_python(source) {
        Ok(function_calls) => {
            println!("Parsed function calls: {:?}", function_calls);
        }
        Err((msg, span)) => {
            let error_message = format!("Error at position {}-{}: {}", span.start, span.end, msg);
            eprintln!("{}", error_message);
        }
    }
}