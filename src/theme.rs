//! Themes: the six colours the switcher is drawn in, from which it works out
//! every other.
//!
//! A theme is a file of colours, picked by name: one of the user's own from
//! `~/.config/jaunt/themes`, or one of those Jaunt comes with. The
//! configuration can also write one out in place, as a table. A colour a
//! theme leaves out is that of the theme it builds on, its `base`, or else
//! the default theme's.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};

/// Each theme by name, with the text of `themes/<name>.toml`.
///
/// A macro rather than a function, because `include_str!` needs its path
/// before any function runs.
macro_rules! themes {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../themes/", $name, ".toml")))),*]
    };
}

/// The theme a colour falls back to, and what `theme` means when the
/// configuration doesn't say.
const DEFAULT: &str = "default";

/// The themes Jaunt comes with, by name.
const BUILT_IN: &[(&str, &str)] = themes![
    // keep-sorted start
    "catppuccin-frappe",
    "catppuccin-latte",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "default",
    "light",
    "nord",
    "one-dark",
    "tokyo-night",
    // keep-sorted end
];

/// The six colours the switcher is drawn in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Palette {
    /// The panel everything lies on.
    background: Colour,
    /// The sheet the windows lie on, and the tab joining it to their
    /// application.
    surface: Colour,
    /// The window a switch would land on.
    accent: Colour,
    /// Window titles, and the keys.
    text: Colour,
    /// The application in the heading, the selected window's title and its
    /// dot.
    bright: Colour,
    /// The window in the heading, what the keys do, and the other windows'
    /// dots.
    muted: Colour,
}

impl Default for Palette {
    fn default() -> Self {
        let (_, text) = BUILT_IN
            .iter()
            .find(|(name, _)| *name == DEFAULT)
            .expect("Jaunt ships a theme called 'default'");

        toml::from_str(text).expect("Jaunt's default theme sets every colour")
    }
}

/// A theme as it's written, in its own file or in the configuration: the
/// theme it builds on, and the colours it changes, which needn't be all of
/// them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Theme {
    /// The theme a colour this one leaves out comes from, by name or path.
    /// Without one, that's Jaunt's default theme.
    base: Option<String>,
    background: Option<Colour>,
    surface: Option<Colour>,
    accent: Option<Colour>,
    text: Option<Colour>,
    bright: Option<Colour>,
    muted: Option<Colour>,
}

impl Theme {
    /// The theme called `name`, just as it is.
    pub(crate) fn named(name: &str) -> Self {
        Self {
            base: Some(name.to_owned()),
            background: None,
            surface: None,
            accent: None,
            text: None,
            bright: None,
            muted: None,
        }
    }

    /// The colours this theme sets, and its base's where it doesn't.
    /// `reading` is every theme already read on the way here, so that one
    /// building on itself is an error rather than a loop.
    fn palette(&self, directory: Option<&Path>, reading: &mut Vec<String>) -> Result<Palette> {
        let base = match &self.base {
            None => Palette::default(),
            Some(name) => {
                if reading.contains(name) {
                    anyhow::bail!("the theme '{name}' builds on itself");
                }

                reading.push(name.clone());

                let (text, source) = find(name, directory)?;
                let theme: Theme =
                    toml::from_str(&text).with_context(|| format!("{source} isn't valid"))?;

                theme.palette(directory, reading)?
            }
        };

        Ok(self.over(base))
    }

    /// The colours this theme sets, and `base`'s where it doesn't.
    fn over(&self, base: Palette) -> Palette {
        Palette {
            background: self.background.clone().unwrap_or(base.background),
            surface: self.surface.clone().unwrap_or(base.surface),
            accent: self.accent.clone().unwrap_or(base.accent),
            text: self.text.clone().unwrap_or(base.text),
            bright: self.bright.clone().unwrap_or(base.bright),
            muted: self.muted.clone().unwrap_or(base.muted),
        }
    }
}

/// Reads the configuration's `theme`, which is either a theme's name or a
/// table written like a theme file.
///
/// Rather than `#[serde(untagged)]`, which can only say that neither fits, so
/// that a misspelt colour is still named as one.
///
/// # Errors
///
/// Returns an error if it's neither, or if the table isn't a theme.
pub(crate) fn named_or_written<'de, D>(deserializer: D) -> Result<Theme, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Theme;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("the name of a theme, or a table of its colours")
        }

        fn visit_str<E>(self, name: &str) -> Result<Theme, E> {
            Ok(Theme::named(name))
        }

        fn visit_map<A>(self, map: A) -> Result<Theme, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            Theme::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(Visitor)
}

/// A colour, written `#rrggbb`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct Colour(String);

impl TryFrom<String> for Colour {
    type Error = String;

    fn try_from(colour: String) -> Result<Self, Self::Error> {
        let digits = colour.strip_prefix('#').unwrap_or_default();

        if digits.len() != 6 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
            return Err(format!(
                "'{colour}' isn't a colour: write it as #rrggbb, like #6b8cff"
            ));
        }

        Ok(Self(colour.to_lowercase()))
    }
}

/// The colours of `theme`, and of the themes it builds on. Those are found
/// by name: the user's own by that name, from `directory`, or else the one
/// Jaunt comes with. A name with a `/` in it is the path to a theme file
/// instead.
///
/// # Errors
///
/// Returns an error if there's no theme by one of those names, if its file
/// can't be read or isn't a theme, or if a theme builds on itself.
pub(crate) fn load(theme: &Theme, directory: Option<&Path>) -> Result<Palette> {
    theme.palette(directory, &mut Vec::new())
}

/// The text of the theme called `name`, and what to call where it came from.
fn find(name: &str, directory: Option<&Path>) -> Result<(String, String)> {
    let path = if name.contains('/') {
        Some(PathBuf::from(name))
    } else {
        directory.map(|directory| directory.join(format!("{name}.toml")))
    };

    if let Some(path) = &path {
        match std::fs::read_to_string(path) {
            Ok(text) => return Ok((text, path.display().to_string())),
            // Not one of the user's own, but it may be one Jaunt comes with.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !name.contains('/') => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to read the theme {}", path.display()));
            }
        }
    }

    let (_, text) = BUILT_IN
        .iter()
        .find(|(built_in, _)| *built_in == name)
        .with_context(|| {
            let built_in: Vec<_> = BUILT_IN.iter().map(|(name, _)| *name).collect();
            let looked = path.map_or_else(String::new, |path| {
                format!("there's no {}, and ", path.display())
            });

            format!(
                "there's no theme called '{name}': {looked}Jaunt's own are {}",
                built_in.join(", ")
            )
        })?;

    Ok(((*text).to_owned(), format!("jaunt's {name} theme")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory of themes for one test, emptied first.
    fn themes(test: &str, files: &[(&str, &str)]) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("jaunt-themes-{test}"));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("failed to make a themes directory");

        for (name, text) in files {
            std::fs::write(directory.join(name), text).expect("failed to write a theme");
        }

        directory
    }

    /// The colours of the theme called `name`.
    fn named(name: &str, directory: Option<&Path>) -> Result<Palette> {
        load(&Theme::named(name), directory)
    }

    fn colour(colour: &str) -> Colour {
        colour.to_owned().try_into().expect("a colour")
    }

    #[test]
    fn the_themes_jaunt_comes_with_set_every_colour() {
        for (name, text) in BUILT_IN {
            toml::from_str::<Palette>(text).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }

    #[test]
    fn a_theme_is_found_by_name_without_any_of_the_users_own() {
        assert_eq!(named("default", None).unwrap(), Palette::default());

        let light = named("light", None).unwrap();
        assert_ne!(light, Palette::default());
        assert_eq!(named("light", Some(&themes("none", &[]))).unwrap(), light);
    }

    #[test]
    fn the_users_own_theme_comes_first_and_falls_back_colour_by_colour() {
        let directory = themes("own", &[("light.toml", "accent = \"#FF0000\"")]);
        let palette = named("light", Some(&directory)).unwrap();

        assert_eq!(palette.accent, colour("#ff0000"));
        // The rest is the default theme's, not the built-in light theme's.
        assert_eq!(palette.background, Palette::default().background);
    }

    #[test]
    fn a_theme_may_be_a_path() {
        let directory = themes("path", &[("mine.toml", "muted = \"#123456\"")]);
        let path = directory.join("mine.toml");

        let palette = named(path.to_str().unwrap(), None).unwrap();
        assert_eq!(palette.muted, colour("#123456"));

        let error = named(directory.join("missing.toml").to_str().unwrap(), None)
            .expect_err("a path that isn't there is an error");
        assert!(format!("{error:#}").contains("missing.toml"), "{error:#}");
    }

    #[test]
    fn a_theme_can_build_on_another_and_that_on_another() {
        let directory = themes(
            "base",
            &[
                ("dusk.toml", "base = \"nord\"\naccent = \"#ff8800\""),
                ("dawn.toml", "base = \"dusk\"\nmuted = \"#123456\""),
            ],
        );
        let nord = named("nord", None).unwrap();
        let dawn = named("dawn", Some(&directory)).unwrap();

        assert_eq!(dawn.muted, colour("#123456"));
        assert_eq!(dawn.accent, colour("#ff8800"));
        assert_eq!(dawn.background, nord.background);
        assert_ne!(dawn.background, Palette::default().background);
    }

    #[test]
    fn a_theme_that_builds_on_itself_is_an_error() {
        let directory = themes(
            "itself",
            &[
                ("nord.toml", "base = \"nord\""),
                ("one.toml", "base = \"two\""),
                ("two.toml", "base = \"one\""),
            ],
        );

        let error = format!("{:#}", named("nord", Some(&directory)).unwrap_err());
        assert!(error.contains("'nord' builds on itself"), "{error}");

        let error = format!("{:#}", named("one", Some(&directory)).unwrap_err());
        assert!(error.contains("'one' builds on itself"), "{error}");
    }

    #[test]
    fn a_theme_that_isnt_there_says_where_jaunt_looked() {
        let directory = themes("missing", &[]);
        let error = named("dracula", Some(&directory)).expect_err("there's no dracula");
        let error = format!("{error:#}");

        assert!(error.contains("dracula.toml"), "{error}");

        // However the list is ordered, it names the ones there are.
        for name in ["default", "light", "catppuccin-mocha", "nord"] {
            assert!(error.contains(name), "{name} is missing from: {error}");
        }
    }

    #[test]
    fn a_typo_in_a_theme_is_an_error() {
        let directory = themes(
            "typo",
            &[
                ("key.toml", "backgorund = \"#000000\""),
                ("colour.toml", "background = \"black\""),
                ("short.toml", "background = \"#000\""),
            ],
        );

        let error = format!("{:#}", named("key", Some(&directory)).unwrap_err());
        assert!(error.contains("backgorund"), "{error}");

        let error = format!("{:#}", named("colour", Some(&directory)).unwrap_err());
        assert!(error.contains("#rrggbb"), "{error}");

        assert!(named("short", Some(&directory)).is_err());
    }
}
