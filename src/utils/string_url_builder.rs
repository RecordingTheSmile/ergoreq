pub trait StringUrlBuilderTrait {
    /// Add a segment to the end of the URL. If the URL ends with `/`, the segment will be added directly. Otherwise, a `/` will be added before the segment.
    /// # Example
    /// ```Rust
    /// let url = "https://example.com";
    /// let segment = "test";
    /// assert_eq!(url.add_url_segment(segment), "https://example.com/test");
    ///
    /// let url = "https://example.com/";
    /// let segment = "test";
    /// assert_eq!(url.add_url_segment(segment), "https://example.com/test");
    ///
    /// let url = "https://example.com";
    /// let segment = "/test";
    /// assert_eq!(url.add_url_segment(segment), "https://example.com/test");
    ///
    /// let url = "https://example.com/";
    /// let segment = "/test";
    /// assert_eq!(url.add_url_segment(segment), "https://example.com/test");
    ///
    /// let url = "https://example.com?query=1";
    /// let segment = "test";
    /// assert_eq!(url.add_url_segment(segment), "https://example.com/test?query=1");
    ///
    /// ```
    fn add_url_segment(self, segment: &str) -> String;
    /// Add multiple segments to the end of the URL. The segments will be added in order.
    /// # Example
    /// ```Rust
    /// let url = "https://example.com";
    /// let segments = &["test", "test1"];
    /// assert_eq!(url.add_url_segments(segments), "https://example.com/test/test1");
    ///
    /// let url = "https://example.com/";
    /// let segments = &["test", "test1"];
    /// assert_eq!(url.add_url_segments(segments), "https://example.com/test/test1");
    ///
    /// let url = "https://example.com";
    /// let segments = &["/test", "/test1"];
    /// assert_eq!(url.add_url_segments(segments), "https://example.com/test/test1");
    ///
    /// let url = "https://example.com/";
    /// let segments = &["/test", "/test1"];
    /// assert_eq!(url.add_url_segments(segments), "https://example.com/test/test1");
    ///
    /// let url = "https://example.com?query=1";
    /// let segments = &["test", "test1"];
    /// assert_eq!(url.add_url_segments(segments), "https://example.com/test/test1?query=1");
    ///
    /// ```
    fn add_url_segments(self, segments: &[&str]) -> String;
}

impl StringUrlBuilderTrait for String {
    fn add_url_segment(self, segment: &str) -> String {
        let query_split = self.split_once("?");

        let segment = segment.trim_start_matches('/');

        if let Some((url, query)) = query_split {
            if url.ends_with("/") {
                if query.is_empty() {
                    format!("{}{}", url, segment)
                } else {
                    format!("{}{}?{}", url, segment, query)
                }
            } else {
                if query.is_empty() {
                    format!("{}/{}", url, segment)
                } else {
                    format!("{}/{}?{}", url, segment, query)
                }
            }
        } else {
            if self.ends_with("/") {
                format!("{}{}", self, segment)
            } else {
                format!("{}/{}", self, segment)
            }
        }
    }

    fn add_url_segments(self, segments: &[&str]) -> String {
        let mut url = self;

        for segment in segments {
            url = url.add_url_segment(segment);
        }

        url
    }
}

impl StringUrlBuilderTrait for &str {
    fn add_url_segment(self, segment: &str) -> String {
        let query_split = self.split_once("?");

        let segment = segment.trim_start_matches('/');

        if let Some((url, query)) = query_split {
            if url.ends_with("/") {
                if query.is_empty() {
                    format!("{}{}", url, segment)
                } else {
                    format!("{}{}?{}", url, segment, query)
                }
            } else {
                if query.is_empty() {
                    format!("{}/{}", url, segment)
                } else {
                    format!("{}/{}?{}", url, segment, query)
                }
            }
        } else {
            if self.ends_with("/") {
                format!("{}{}", self, segment)
            } else {
                format!("{}/{}", self, segment)
            }
        }
    }

    fn add_url_segments(self, segments: &[&str]) -> String {
        let mut url = self.to_owned();

        for segment in segments {
            url = url.add_url_segment(segment);
        }

        url
    }
}

#[cfg(test)]
mod test_string_url_builder {
    use super::StringUrlBuilderTrait;

    #[test]
    fn test_add_url_segment() {
        let url = "https://example.com";
        let segment = "test";

        assert_eq!(url.add_url_segment(segment), "https://example.com/test");

        let url = "https://example.com/";
        let segment = "test";

        assert_eq!(url.add_url_segment(segment), "https://example.com/test");

        let url = "https://example.com";
        let segment = "/test";

        assert_eq!(url.add_url_segment(segment), "https://example.com/test");

        let url = "https://example.com/";
        let segment = "/test";

        assert_eq!(url.add_url_segment(segment), "https://example.com/test");

        let url = "https://example.com?query=1";
        let segment = "test";

        assert_eq!(
            url.add_url_segment(segment),
            "https://example.com/test?query=1"
        );

        let url = "https://example.com/?query=1";
        let segment = "test";

        assert_eq!(
            url.add_url_segment(segment),
            "https://example.com/test?query=1"
        );

        let url = "https://example.com?query=1";
        let segment = "/test";

        assert_eq!(
            url.add_url_segment(segment),
            "https://example.com/test?query=1"
        );

        let url = "https://example.com/?query=1";
        let segment = "/test";

        assert_eq!(
            url.add_url_segment(segment),
            "https://example.com/test?query=1"
        );
    }

    #[test]
    fn test_add_url_segments() {
        let url = "https://example.com";
        let segments = &["test", "test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1"
        );

        let url = "https://example.com/";
        let segments = &["test", "test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1"
        );

        let url = "https://example.com";
        let segments = &["/test", "/test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1"
        );

        let url = "https://example.com/";
        let segments = &["/test", "/test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1"
        );

        let url = "https://example.com?query=1";
        let segments = &["test", "test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1?query=1"
        );

        let url = "https://example.com/?query=1";
        let segments = &["test", "test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1?query=1"
        );

        let url = "https://example.com?query=1";
        let segments = &["/test", "/test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1?query=1"
        );

        let url = "https://example.com/?query=1";
        let segments = &["/test", "/test1"];

        assert_eq!(
            url.add_url_segments(segments),
            "https://example.com/test/test1?query=1"
        );
    }
}
