use proc_macro2::{TokenStream as TokenStream2};
use quote::ToTokens;
use crate::attr_parser::{PropAttr, parse_prop_attr};
use syn::{Path, Error, Type, Field, FnArg, Attribute, Pat};
use syn::spanned::Spanned;

#[derive(Clone)]
pub(crate) struct TypeToInject {
    pub(crate) type_name: String,
    pub(crate) type_path: Path,
    pub(crate) arg_name: Option<TokenStream2>,
    pub(crate) prop_attr: Option<PropAttr>
}


impl TypeToInject {
    pub(crate) fn from_type(type_: &Type) -> Result<Self, Error> {
        Ok(Self {
            type_name: type_.to_token_stream().to_string(),
            type_path: Self::parse_path(type_)?,
            arg_name: None,
            prop_attr: None
        })
    }
    pub(crate) fn from_field(field: &Field) -> Result<Self, Error> {
        Ok(Self {
            type_name: field.ty.to_token_stream().to_string(),
            type_path: Self::parse_path(&field.ty)?,
            arg_name: field.ident.clone().map(|name| name.to_token_stream()),
            prop_attr: Self::parse_attr(&field.attrs)?
        })
    }
    pub(crate) fn from_fn_arg(arg: FnArg) -> Result<Self, Error> {
        let typed = if let FnArg::Typed(typed) = &arg {
            typed
        } else {
            return Err(Error::new(arg.span(), "Unsupported argument type"))
        };

        let arg_name = if let Pat::Ident(pat_ident) = *typed.pat.clone() {
            Some(pat_ident.ident.to_token_stream())
        } else {
            return Err(Error::new(arg.span(), "Unsupported argument name"))
        };

        Ok(Self {
            type_name: typed.ty.to_token_stream().to_string(),
            type_path: Self::parse_path(&typed.ty)?,
            arg_name,
            prop_attr: Self::parse_attr(&typed.attrs)?
        })
    }

    fn parse_attr(attrs: &Vec<Attribute>) -> Result<Option<PropAttr>, Error> {
        let prop_attr = attrs.iter()
            .find(|attr| attr.path.to_token_stream().to_string() == "prop".to_string());

        if prop_attr.is_some() {
            return parse_prop_attr(prop_attr.unwrap())
                .map(|parsed| Some(parsed));
        }
        Ok(None)
    }

    fn parse_path(type_: &Type) -> Result<Path, Error> {
        if let Type::Path(path_type) = type_ {
            Ok(path_type.path.clone())
        } else {
            Err(Error::new(type_.span(), "Unsupported type"))
        }
    }
}
