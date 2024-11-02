use regex::Regex;

pub fn is_email(email: &str) -> bool {
    let re = Regex::new(r"^[\w.-]+@([\w-]+\.)+[a-zA-Z]{2,4}$").unwrap();
    re.is_match(email)
}
