// Jetporch
// Copyright (C) 2023 - Jetporch Project Contributors
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// long with this program.  If not, see <http://www.gnu.org/licenses/>.

use handlebars::{Handlebars, RenderError, HelperDef, RenderContext, ScopedJson, JsonValue, Helper, Context, handlebars_helper};

//#[allow(non_camel_case_types)]
pub struct IsDefined;

impl HelperDef for IsDefined {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let params = h.params();
        if params.len() != 1 {
            return Err(RenderError::new(
                "is_defined: requires one parameter".to_owned(),
            ));
        }
        let result = h.param(0)
            .and_then(|x| {
                if x.is_value_missing() {
                    Some(false)
                } else {
                    Some(true)
                }
            })
            .ok_or_else(|| RenderError::new("is_defined: Couldn't read parameter".to_owned()))?;

        Ok(ScopedJson::Derived(JsonValue::from(result)))
    }
}

pub fn register_helpers(handlebars: &mut Handlebars) {
    {
        handlebars_helper!(to_lower_case: |v: str| v.to_lowercase());
        handlebars.register_helper("to_lower_case", Box::new(to_lower_case))
    }
    {
        handlebars_helper!(to_upper_case: |v: str| v.to_uppercase());
        handlebars.register_helper("to_upper_case", Box::new(to_upper_case))
    }
    {
        handlebars_helper!(trim: |v: str| v.trim());
        handlebars.register_helper("trim", Box::new(trim))
    }
    {
        handlebars_helper!(trim_start: |v: str| v.trim_start());
        handlebars.register_helper("trim_start", Box::new(trim_start))
    }
    {
        handlebars_helper!(trim_end: |v: str| v.trim_end());
        handlebars.register_helper("trim_end", Box::new(trim_end))
    }
    {
        handlebars_helper!(contains: |v: str, s: str| v.contains(s));
        handlebars.register_helper("contains", Box::new(contains))
    }
    {
        handlebars_helper!(starts_with: |v: str, s: str| v.starts_with(s));
        handlebars.register_helper("starts_with", Box::new(starts_with))
    }
    {
        handlebars_helper!(ends_with: |v: str, s: str| v.ends_with(s));
        handlebars.register_helper("ends_with", Box::new(ends_with))
    }
    {
        handlebars.register_helper("isdefined", Box::new(IsDefined));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use serde_json::json;

    use handlebars::{no_escape, Handlebars};

    pub fn new_handlebars<'reg>() -> Handlebars<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(no_escape); //html escaping is the default and cause issue
        register_helpers(&mut handlebars);
        handlebars
    }

    #[macro_export]
    macro_rules! assert_renders {
        ($($arg:expr),+$(,)?) => {{
            use std::collections::HashMap;
            let vs: HashMap<String, String> = HashMap::new();
            let mut handlebars = new_handlebars();
            $({
                let sample: (&str, &str) = $arg;
                handlebars.register_template_string(&sample.0, &sample.0).expect("register_template_string");
                assert_eq!(handlebars.render(&sample.0, &vs).expect("render"), sample.1.to_owned());
            })*
            Ok(())
        }}
    }

    fn test_condition(condition: &str, expected: bool) {
        let handlebars = new_handlebars();
        let result = handlebars
            .render_template(
                &format!(
                    "{{{{#if {condition}}}}}lorem{{{{else}}}}ipsum{{{{/if}}}}",
                    condition = condition
                ),
                &json!({}),
            )
            .unwrap();
        assert_eq!(&result, if expected { "lorem" } else { "ipsum" }, "testing condition: {}", condition);
    }

    #[test]
    fn test_register_string_helpers() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ to_lower_case "Hello foo-bars" }}"##, r##"hello foo-bars"##),
            (r##"{{ to_upper_case "Hello foo-bars" }}"##, r##"HELLO FOO-BARS"##)
        ]
    }

    #[test]
    fn test_helper_trim() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ trim "foo" }}"##, r##"foo"##),
            (r##"{{ trim "  foo" }}"##, r##"foo"##),
            (r##"{{ trim "foo  " }}"##, r##"foo"##),
            (r##"{{ trim " foo " }}"##, r##"foo"##)
        ]
    }

    #[test]
    fn test_helper_trim_start() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ trim_start "foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "  foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "foo  " }}"##, r##"foo  "##),
            (r##"{{ trim_start " foo " }}"##, r##"foo "##)
        ]
    }

    #[test]
    fn test_helper_contains() -> Result<(), Box<dyn Error>> {
        test_condition(r#"( contains "foo" "bar" )"#, false);
        test_condition(r#"( contains "foo" "foo" )"#, true);
        test_condition(r#"( contains "barfoobar" "foo" )"#, true);
        test_condition(r#"( contains "foo" "barfoobar" )"#, false);

        Ok(())
    }

    #[test]
    fn test_helper_starts_with() -> Result<(), Box<dyn Error>> {
        test_condition(r#"( starts_with "foo" "bar" )"#, false);
        test_condition(r#"( starts_with "foobar" "foo" )"#, true);
        test_condition(r#"( starts_with "foo" "foobar" )"#, false);

        Ok(())
    }

    #[test]
    fn test_helper_ends_with() -> Result<(), Box<dyn Error>> {
        test_condition(r#"( ends_with "foo" "bar" )"#, false);
        test_condition(r#"( ends_with "foobar" "bar" )"#, true);
        test_condition(r#"( ends_with "foo" "foobar" )"#, false);

        Ok(())
    }

    #[test]
    fn test_isdefined_none() -> Result<(), Box<dyn Error>> {
        let handlebars = new_handlebars();

        let result = handlebars.render_template(
            r#"{{isdefined a}} {{isdefined b}} {{#if (isdefined a)}}a{{/if}} {{#if (isdefined b)}}b{{/if}}"#,
            &json!({})
        );
        assert_eq!(result.unwrap(), "false false  ");
        Ok(())
    }

    #[test]
    fn test_isdefined_a_and_b() -> Result<(), Box<dyn Error>> {
        let handlebars = new_handlebars();

        let result = handlebars.render_template(
            r#"{{isdefined a}} {{isdefined b}} {{#if (isdefined a)}}a{{/if}} {{#if (isdefined b)}}b{{/if}}"#,
            &json!({"a": 1, "b": 2})
        );
        assert_eq!(result.unwrap(), "true true a b");
        Ok(())
    }

    #[test]
    fn test_isdefined_a() -> Result<(), Box<dyn Error>> {
        let handlebars = new_handlebars();

        let result = handlebars.render_template(
            r#"{{isdefined a}} {{isdefined b}} {{#if (isdefined a)}}a{{/if}} {{#if (isdefined b)}}b{{/if}}"#,
            &json!({"a": 1})
        );
        assert_eq!(result.unwrap(), "true false a ");
        Ok(())
    }
}
