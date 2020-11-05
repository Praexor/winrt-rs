use crate::*;
use squote::{quote, Literal, TokenStream};

#[derive(Debug)]
pub struct Struct {
    pub name: TypeName,
    pub fields: Vec<(String, TypeKind)>, // TODO: might have to be a full Type to ensure we can write out nested structs for ABI layout
    pub signature: String,
}

impl Struct {
    pub fn from_type_name(reader: &winmd::TypeReader, name: TypeName) -> Self {
        let signature = name.struct_signature(reader);
        let mut fields = Vec::new();

        for field in name.def.fields(reader) {
            let field_name = to_snake(field.name(reader), MethodKind::Normal);
            let kind = TypeKind::from_field(reader, field, &name.namespace);
            fields.push((field_name, kind));
        }

        Self {
            name,
            fields,
            signature,
        }
    }

    pub fn dependencies(&self) -> Vec<winmd::TypeDef> {
        self.fields
            .iter()
            .flat_map(|i| i.1.dependencies())
            .collect()
    }

    pub fn gen(&self) -> TokenStream {
        let name = self.name.gen();
        let signature = Literal::byte_string(&self.signature.as_bytes());

        // TODO: any struct fields that have COM as underlying type must be wrapped in Option<T>

        let fields = self.fields.iter().map(|field| {
            let name = format_ident(&field.0);
            let kind = field.1.gen();
            quote! {
                pub #name: #kind
            }
        });

        quote! {
            #[repr(C)]
            #[derive(::std::clone::Clone, ::std::default::Default, ::std::cmp::PartialEq)]
            pub struct #name {
                #(#fields),*
            }
            unsafe impl ::winrt::RuntimeType for #name {
                const SIGNATURE: ::winrt::ConstBuffer = ::winrt::ConstBuffer::from_slice(#signature);
            }
            unsafe impl ::winrt::Abi for #name {
                type Abi = Self;
                unsafe fn get_abi(&self) -> Self {
                    self.clone()
                }
                unsafe fn set_abi(&mut self) -> *mut Self {
                    self
                }

            }
            unsafe impl ::winrt::IntoResult for #name {
                type Abi = Self;
                unsafe fn into_result(abi: Self::Abi) -> ::winrt::Result<Self> {
                    Ok(abi)
                }
            }
        }
    }
}
