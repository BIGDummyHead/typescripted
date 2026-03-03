use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

/// # Export TypeScript
///
/// Exports the Struct or Enum to a TypeScript (.ts) file.
///
/// The files that are generated are stored in `./types/*.ts`
///
/// The following example generates the directory `types` then *2* files `User.ts` and `Role.ts`.
///
/// It is important to note that when generated it is assumed that Role will ALSO be exported.
///
/// Usage:
///
/// ```
/// #[derive(ExportTypescript)]
/// struct User {
///     pub uid: i32;
///     pub display_name: string;
///     pub role: Role;
/// }
///
/// #[derive(ExportTypescript)]
/// enum Role {
///     User,
///     Admin
/// }
/// ```
///
/// Generated data:
///
/// User.ts:
///
/// ```
/// import Role from './Role.ts';
///
/// interface User {
///     uid: number;
///     display_name: string;
///     role: Role;
/// }
///
/// export default User;
///
/// ```
///
/// Role.ts:
///
/// ```
/// enum Role {
///     User,
///     Admin
/// }
///
/// export default Role;
/// ```
#[proc_macro_derive(ExportTypescript)]
pub fn export_typescript(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Converter for Rust basic types to TypeScript types.
    // Custom types can be still be used as they rely on other interfaces.
    let mut type_converter = HashMap::new();
    type_converter.insert("i8", "number");
    type_converter.insert("i16", "number");
    type_converter.insert("i32", "number");
    type_converter.insert("i64", "number");
    type_converter.insert("i128", "number");
    type_converter.insert("isize", "number");
    type_converter.insert("u8", "number");
    type_converter.insert("u16", "number");
    type_converter.insert("u32", "number");
    type_converter.insert("u64", "number");
    type_converter.insert("u128", "number");
    type_converter.insert("usize", "number");
    type_converter.insert("f32", "number");
    type_converter.insert("f64", "number");
    type_converter.insert("bool", "boolean");
    type_converter.insert("String", "string");
    type_converter.insert("str", "string");
    type_converter.insert("()", "void");
    type_converter.insert("Vec", "Array");
    type_converter.insert("Option", "any");
    type_converter.insert("Result", "any");
    type_converter.insert("HashMap", "object");
    type_converter.insert("BTreeMap", "object");

    // a maps each field into a Vec<String> containing their field names.
    let mut field_data_string: String = match input.data {
        Data::Struct(ds) => {
            //export interface NAME {
            let mut interface_export_output: String = "interface ".to_string();
            interface_export_output.push_str(input.ident.to_string().as_str());
            interface_export_output.push_str(" {\r\n");

            //imports
            let mut imports: HashMap<String, String> = HashMap::new();

            let field_types = ds.fields.into_iter().map(|f| quote!(#f.ty).to_string());

            let mut output_types: Vec<String> = Vec::new();
            for field_type in field_types {
                //empty or non public
                if field_type.is_empty() || !field_type.contains("pub") {
                    continue;
                }

                // [documentXXXX] pub
                let field_type = field_type
                    .trim()
                    .rsplit_once("pub")
                    .map(|(_, r)| {
                        r.split_once(":")
                            .map(|(l, r)| (l.trim(), r.trim()))
                            .unwrap_or(("", ""))
                    })
                    .and_then(|(l, r)| {
                        if l.is_empty() || r.is_empty() {
                            None
                        } else {
                            let (ty_name, _) = r.split_once(".").unwrap();

                            Some((l.trim(), ty_name.trim()))
                        }
                    });

                //nothing to do
                if field_type.is_none() {
                    continue;
                }

                let (name, ty) = field_type.unwrap();

                //convert our unsanitized Rust type to TypeScript.
                let formal_name = if let Some(formal) = type_converter.get(ty) {
                    formal.to_string()
                } else {
                    //import { X } from './X';
                    let import_value = format!("import {} from './{}';", ty, ty);
                    imports.insert(ty.to_string(), import_value);

                    ty.to_string()
                };

                output_types.push(format!("\t{name}: {formal_name};"));
            }

            let output_types = output_types.join("\r\n");
            interface_export_output.push_str(&output_types);
            interface_export_output.push_str("\r\n}");

            let mut output = String::new();

            for (_, import) in &imports {
                output.push_str(&format!("{import}\r\n"));
            }

            output.push_str("\r\n");
            output.push_str(&interface_export_output);

            output
        }
        Data::Enum(de) => {
            let mut output = String::new();

            let enum_name = input.ident.to_string();
            output.push_str(&format!("enum {enum_name} {{\r\n"));

            //identifiers within the enum
            let fields: Vec<String> = de
                .variants
                .into_iter()
                .map(|f| quote!(#f.ident).to_string())
                .map(|s| {
                    //remove the #doc comments and other macros
                    let (_, name) = s.rsplit_once("]").unwrap();

                    name.trim().to_string()
                })
                .map(|s| {
                    let spl = s.split_once(".");

                    if let Some((left, _)) = spl {
                        let left = left.trim();
                        format!("\t{left} = \"{left}\"")
                    } else {
                        "".to_string()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();

            let fields = fields.join(",\r\n");

            output.push_str(&fields);

            output.push_str("\r\n}");

            output
        }
        _ => panic!("this is not yet supported."),
    };

    //convert field data into vec<u8>
    field_data_string.push_str("\r\n\r\nexport default ");
    field_data_string.push_str(&format!("{}", input.ident.to_string()));
    let output_data = field_data_string.into_bytes();

    let path = format!("./types/{}.ts", input.ident.to_string());

    let _ = fs::create_dir("./types");

    File::create(&path)
        .unwrap_or_else(|e| panic!("{e}, could not open"))
        .write_all(&output_data)
        .unwrap_or_else(|e| panic!("{e}, could not write"));

    //do nothing.  data collected and wrote
    TokenStream::new()
}
