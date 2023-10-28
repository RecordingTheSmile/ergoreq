use cookie::Cookie;

pub struct ErgoCookieParser;

impl ErgoCookieParser {
    /// Parse all `set-cookie` header.
    ///
    /// # Error
    /// All `set-cookie` headers which contains errors will be **Ignored**
    pub fn parse_set_cookie_header<'a, S>(headers: S) -> Vec<Cookie<'a>>
    where
        S: Iterator<Item = &'a str>,
    {
        headers
            .into_iter()
            .map(Cookie::parse_encoded)
            // drop error `set-cookie` header
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test_cookie_parser {
    const SET_COOKIE_HEADERS: [&str; 13] = [
        "mycookie=example; path=/; domain=",
        "subdomain_cookie=subdomain; path=/; domain=.example.com; domain=example2.com",
        "domain_cookie=domain; path=/; domain=example.com",
        "cross_domain_cookie=cross; path=/; domain=example.com; domain=example2.com",
        "session=abc123; path=/",
        "user=johndoe; path=/profile",
        "lang=en-US; expires=Thu, 28 Oct 2023 14:30:00 GMT",
        "theme=dark; domain=example.com",
        "remember=true; path=/; secure",
        "deleted=; expires=Thu, 01 Jan 1970 00:00:00 GMT",
        "httpOnly=true; path=/; HttpOnly",
        "maxAgeCookie=test; path=/; max-age=3600",
        "sameSiteCookie=test; path=/; SameSite=Strict",
    ];

    #[test]
    fn test_parse_cookie_header() {
        use super::ErgoCookieParser;

        let cookies = ErgoCookieParser::parse_set_cookie_header(SET_COOKIE_HEADERS.into_iter());
        assert_eq!(cookies.len(), 13);
    }
}
