use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Result};
use serde_json::json;

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Nullable_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        let mut schema = if gen.settings().option_nullable {
            T::json_schema(gen)?
        } else {
            gen.subschema_for::<T>()?
        };
        if gen.settings().option_add_null_type {
            schema = match schema {
                Schema::Bool(true) => Schema::Bool(true),
                Schema::Bool(false) => <()>::json_schema(gen)?,
                schema => SchemaObject {
                    any_of: Some(vec![schema, <()>::json_schema(gen)?]),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut deref = gen.get_schema_object(schema)?;
            deref.extensions.insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(deref);
        };
        Ok(schema)
    }
}

impl<T: ?Sized> JsonSchema for std::marker::PhantomData<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        <()>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        <()>::json_schema(gen)
    }
}

impl JsonSchema for std::convert::Infallible {
    no_ref_schema!();

    fn schema_name() -> String {
        "Never".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        Ok(gen.schema_for_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::*;
    use crate::tests::{custom_schema_object_for, schema_for, schema_object_for};
    use pretty_assertions::assert_eq;

    #[test]
    fn schema_for_option() {
        let schema = schema_object_for::<Option<i32>>();
        assert_eq!(schema.instance_type, None);
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.any_of.is_some(), true);
        let any_of = schema.any_of.unwrap();
        assert_eq!(any_of.len(), 2);
        assert_eq!(any_of[0], schema_for::<i32>());
        assert_eq!(any_of[1], schema_for::<()>());
    }

    #[test]
    fn schema_for_option_with_nullable() {
        let settings = SchemaSettings {
            option_nullable: true,
            option_add_null_type: false,
            ..Default::default()
        };
        let schema = custom_schema_object_for::<Option<i32>>(settings);
        assert_eq!(
            schema.instance_type,
            Some(SingleOrVec::from(InstanceType::Integer))
        );
        assert_eq!(schema.extensions.get("nullable"), Some(&json!(true)));
        assert_eq!(schema.any_of.is_none(), true);
    }
}
