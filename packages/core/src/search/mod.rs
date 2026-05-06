//! Natural Language Search Engine — KeePassEx
//!
//! Parse human-readable queries in English and Vietnamese into structured
//! vault search filters. No external NLP library needed — pure rule-based
//! tokenizer + intent classifier.
//!
//! # Examples (English)
//! - "show all banking passwords"
//! - "find expired entries with weak passwords"
//! - "entries created last month"
//! - "passwords not used in 6 months"
//! - "entries with OTP in Work group"
//!
//! # Examples (Vietnamese)
//! - "tìm mật khẩu ngân hàng"
//! - "mục đã hết hạn với mật khẩu yếu"
//! - "mục tạo tháng trước"
//! - "mật khẩu chưa dùng 6 tháng"
//! - "mục có OTP trong nhóm Công việc"

pub mod nl_parser;
pub mod query_builder;

pub use nl_parser::{parse_nl_query, NlIntent, NlQuery};
pub use query_builder::{build_search_filter, SearchFilter, SortOrder};
