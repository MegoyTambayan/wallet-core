use core::panic;

use crate::{
    grammar::{GHeaderFileItem, GMarker, GType, GTypeCategory},
    CHeaderDirectory,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    BadImport,
    BadObject,
    BadProperty,
    BadType,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    #[serde(flatten)]
    pub variant: TypeVariant,
    pub is_constant: bool,
    pub is_nullable: bool,
    pub is_pointer: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant", content = "value", rename_all = "snake_case")]
pub enum TypeVariant {
    Void,
    Bool,
    Char,
    ShortInt,
    Int,
    UnsignedInt,
    LongInt,
    Float,
    Double,
    SizeT,
    Int8T,
    Int16T,
    Int32T,
    Int64T,
    UInt8T,
    UInt16T,
    UInt32T,
    UInt64T,
    Struct(String),
    Enum(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub imports: Vec<ImportInfo>,
    pub structs: Vec<StructInfo>,
    pub inits: Vec<InitInfo>,
    pub deinits: Vec<DeinitInfo>,
    pub enums: Vec<EnumInfo>,
    pub functions: Vec<FunctionInfo>,
    pub properties: Vec<PropertyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    // Expressed as directories plus the final file.
    // E.g. `to/some/file.h` ~= ["to", "some", "file.h"]
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub is_public: bool,
    pub variants: Vec<(String, Option<usize>)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub is_public: bool,
    pub fields: Vec<(String, TypeInfo)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitInfo {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeinitInfo {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub is_public: bool,
    pub is_static: bool,
    pub params: Vec<ParamInfo>,
    pub return_type: TypeInfo,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub is_public: bool,
    pub is_static: bool,
    pub return_type: TypeInfo,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: TypeInfo,
}

// NOTE: This function is temporary
pub fn process_c_header_dir(dir: &CHeaderDirectory) {
    for (path, items) in &dir.map {
        let file_name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".h")
            .unwrap()
            .to_string();

        let mut file_info = FileInfo {
            name: file_name.clone(),
            imports: vec![],
            structs: vec![],
            inits: vec![],
            deinits: vec![],
            enums: vec![],
            functions: vec![],
            properties: vec![],
        };

        for item in items {
            match item {
                GHeaderFileItem::HeaderInclude(decl) => {
                    let x = ImportInfo::from_g_type(decl).unwrap();
                    file_info.imports.push(x);
                }
                GHeaderFileItem::StructIndicator(decl) => {
                    file_info.structs.push(StructInfo {
                        name: decl.name.0 .0.clone(),
                        is_public: true,
                        fields: vec![],
                        tags: vec![],
                    });
                }
                GHeaderFileItem::StructDecl(decl) => {
                    let x = StructInfo::from_g_type(decl).unwrap();
                    file_info.structs.push(x);
                }
                GHeaderFileItem::EnumDecl(decl) => {
                    let x = EnumInfo::from_g_type(decl).unwrap();
                    file_info.enums.push(x);
                }
                GHeaderFileItem::FunctionDecl(decl) => {
                    let markers = &decl.markers.0;

                    // Handle exported methods.
                    if markers.contains(&GMarker::TwExportMethod)
                        || markers.contains(&GMarker::TwExportStaticMethod)
                    {
                        // Detect constructor methods.
                        if decl.name.0.contains("Create") {
                            let x = InitInfo::from_g_type(decl).unwrap();
                            file_info.inits.push(x);
                        }
                        // Delect deconstructor methods.
                        else if decl.name.0.contains("Delete") {
                            let x = DeinitInfo::from_g_type(decl).unwrap();
                            file_info.deinits.push(x);
                        }
                        // Any any other method is just a method.
                        else {
                            let x = FunctionInfo::from_g_type(&None, decl).unwrap();
                            file_info.functions.push(x);
                        }
                    }
                    // Handle exported properties.
                    else if markers.contains(&GMarker::TwExportProperty)
                        || markers.contains(&GMarker::TwExportStaticProperty)
                    {
                        let x = PropertyInfo::from_g_type(decl).unwrap();
                        file_info.properties.push(x);
                    }
                    // None-exported methods are skipped.
                    else {
                        println!("Skipped: {}", &decl.name.0);
                    }
                }
                _ => {}
            }
        }

        let content = serde_yaml::to_string(&file_info).unwrap();
        let file_path = format!("out/{}.yaml", file_name);

        std::fs::create_dir_all("out").unwrap();
        std::fs::write(&file_path, content.as_bytes()).unwrap();
    }
}

pub fn extract_custom(ty: &GType) -> Option<String> {
    match ty {
        GType::Mutable(cat) | GType::Const(cat) | GType::Extern(cat) => {
            if let GTypeCategory::Unrecognized(keyword) = cat {
                Some(keyword.0.clone())
            } else {
                None
            }
        }
    }
}

#[test]
#[ignore]
fn test_manifest() {
    let path = std::path::Path::new("../include/");
    let dir = crate::parse(&path).unwrap();
    process_c_header_dir(&dir);
}