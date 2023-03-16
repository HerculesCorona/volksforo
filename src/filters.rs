use chrono::{DateTime, Duration, NaiveDateTime, Utc};

// This filter does not have extra arguments
pub fn duration_timestamp(d: &Duration) -> ::askama::Result<String> {
    if let Some(datetime) = NaiveDateTime::from_timestamp_opt(d.num_seconds(), 0) {
        let time: DateTime<Utc> = DateTime::from_utc(datetime, Utc);
        let human = time.format("%b %e, %Y at %l:%M %p %Z");
        Ok(format!(
            "<time datetime=\"{}\" data-timestamp=\"{}\" title=\"{}\">{}</time>",
            time.format("%Y-%m-%dT%H:%M:%S%z"),
            d.num_seconds(),
            human,
            human
        ))
    } else {
        Ok(format!("<time>{}</time>", d.num_seconds()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        // Formatting errors cause very vague panics that shut down the entire program.
        // NaiveDateTime does NOT have a concept of timezones and %z/%Z cause a panic.
        let d = Duration::seconds(1678983586);
        println!("{}", duration_timestamp(&d).unwrap());
    }
}
