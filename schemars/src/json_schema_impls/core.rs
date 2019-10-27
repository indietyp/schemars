use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use serde_json::json;

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Nullable_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = if gen.settings().option_nullable {
            T::json_schema(gen)
        } else {
            gen.subschema_for::<T>()
        };
        if gen.settings().option_add_null_type {
            schema = match schema {
                Schema::Bool(true) => Schema::Bool(true),
                Schema::Bool(false) => <()>::json_schema(gen),
                Schema::Object(
                    obj @ SchemaObject {
                        instance_type: Some(_),
                        ..
                    },
                ) => Schema::Object(with_null_type(obj)),
                schema => SchemaObject {
                    subschemas: Some(Box::new(SubschemaValidation {
                        any_of: Some(vec![schema, <()>::json_schema(gen)]),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut schema_obj: SchemaObject = schema.into();
            schema_obj
                .extensions
                .insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(schema_obj);
        };
        schema
    }

    fn json_schema_optional(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = T::json_schema_optional(gen);
        if let Schema::Object(SchemaObject {
            object: Some(ref mut object_validation),
            ..
        }) = schema
        {
            object_validation.required.clear();
        }
        schema
    }
}

fn with_null_type(mut obj: SchemaObject) -> SchemaObject {
    match obj
        .instance_type
        .as_mut()
        .expect("checked in calling function")
    {
        SingleOrVec::Single(ty) if **ty == InstanceType::Null => {}
        SingleOrVec::Vec(ty) if ty.contains(&InstanceType::Null) => {}
        SingleOrVec::Single(ty) => obj.instance_type = Some(vec![**ty, InstanceType::Null].into()),
        SingleOrVec::Vec(ref mut ty) => ty.push(InstanceType::Null),
    };
    obj
}

impl<T: ?Sized> JsonSchema for std::marker::PhantomData<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        <()>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <()>::json_schema(gen)
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
        assert_eq!(
            schema.instance_type,
            Some(vec![InstanceType::Integer, InstanceType::Null].into())
        );
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.subschemas.is_none(), true);
    }

    #[test]
    fn schema_for_option_with_ref() {
        use crate as schemars;
        #[derive(JsonSchema)]
        struct Foo;

        let schema = schema_object_for::<Option<Foo>>();
        assert_eq!(schema.instance_type, None);
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.subschemas.is_some(), true);
        let any_of = schema.subschemas.unwrap().any_of.unwrap();
        assert_eq!(any_of.len(), 2);
        assert_eq!(any_of[0], Schema::new_ref("#/definitions/Foo".to_string()));
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
        assert_eq!(schema.subschemas.is_none(), true);
    }
}
