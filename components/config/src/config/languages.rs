use std::collections::HashMap;

use errors::{bail, Result};
use libs::unic_langid::LanguageIdentifier;
use serde::{Deserialize, Serialize};

use crate::config::search;
use crate::config::taxonomies;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LanguageOptions {
    /// Title of the site. Defaults to None
    pub title: Option<String>,
    /// Description of the site. Defaults to None
    pub description: Option<String>,
    /// Whether to generate a feed for that language, defaults to `false`
    pub generate_feed: bool,
    /// The filename to use for feeds. Used to find the template, too.
    /// Defaults to "atom.xml", with "rss.xml" also having a template provided out of the box.
    pub feed_filename: String,
    pub taxonomies: Vec<taxonomies::TaxonomyConfig>,
    /// Whether to generate search index for that language, defaults to `false`
    pub build_search_index: bool,
    /// The search config, telling what to include in the search index for that language
    pub search: search::Search,
    /// A toml crate `Table` with String key representing term and value
    /// another `String` representing its translation.
    /// Use `get_translation()` method for translating key into different languages.
    pub translations: HashMap<String, String>,
}

impl LanguageOptions {
    /// Merges self with another LanguageOptions, erroring if 2 equivalent fields are not None,
    /// empty or the default value.
    pub fn merge(&mut self, other: &LanguageOptions) -> Result<()> {
        macro_rules! merge_field {
            ($orig_field:expr,$other_field:expr,$name:expr) => {
                match &$orig_field {
                    None => $orig_field = $other_field.clone(),
                    Some(cur_value) => {
                        if let Some(other_field_value) = &$other_field {
                            bail!(
                                "`{}` for default language is specified twice, as {:?} and {:?}.",
                                $name,
                                cur_value,
                                other_field_value
                            );
                        }
                    }
                };
            };
            ($cond:expr,$orig_field:expr,$other_field:expr,$name:expr) => {
                if $cond {
                    $orig_field = $other_field.clone();
                } else if !$other_field.is_empty() {
                    bail!(
                        "`{}` for default language is specified twice, as {:?} and {:?}.",
                        $name,
                        $orig_field,
                        $other_field
                    )
                }
            };
        }
        merge_field!(self.title, other.title, "title");
        merge_field!(self.description, other.description, "description");
        merge_field!(
            self.feed_filename == "atom.xml",
            self.feed_filename,
            other.feed_filename,
            "feed_filename"
        );
        merge_field!(self.taxonomies.is_empty(), self.taxonomies, other.taxonomies, "taxonomies");
        merge_field!(
            self.translations.is_empty(),
            self.translations,
            other.translations,
            "translations"
        );

        self.generate_feed = self.generate_feed || other.generate_feed;
        self.build_search_index = self.build_search_index || other.build_search_index;

        if self.search == search::Search::default() {
            self.search = other.search.clone();
        } else if self.search != other.search {
            bail!(
                "`search` for default language is specified twice, as {:?} and {:?}.",
                self.search,
                other.search
            );
        }

        Ok(())
    }
}

/// We want to ensure the language codes are valid ones
pub fn validate_code(code: &str) -> Result<()> {
    if LanguageIdentifier::from_bytes(code.as_bytes()).is_err() {
        bail!("Language `{}` is not a valid Unicode Language Identifier (see http://unicode.org/reports/tr35/#Unicode_language_identifier)", code)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_without_conflict() {
        let mut base_default_language_options = LanguageOptions {
            title: Some("Site's title".to_string()),
            description: None,
            generate_feed: true,
            feed_filename: "atom.xml".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        let section_default_language_options = LanguageOptions {
            title: None,
            description: Some("Site's description".to_string()),
            generate_feed: false,
            feed_filename: "rss.xml".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        base_default_language_options.merge(&section_default_language_options).unwrap();
    }

    #[test]
    fn merge_with_conflict() {
        let mut base_default_language_options = LanguageOptions {
            title: Some("Site's title".to_string()),
            description: Some("Duplicate site description".to_string()),
            generate_feed: true,
            feed_filename: "".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        let section_default_language_options = LanguageOptions {
            title: None,
            description: Some("Site's description".to_string()),
            generate_feed: false,
            feed_filename: "Some feed_filename".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        let res =
            base_default_language_options.merge(&section_default_language_options).unwrap_err();
        assert!(res.to_string().contains("`description` for default language is specified twice"));
    }
}
