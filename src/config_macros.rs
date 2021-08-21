use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(ConfParsable)]
pub fn derive_conf_tree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = syn::parse_macro_input!(input as syn::DeriveInput);

  let toks = conf_tree(&ast).unwrap_or_else(|err| err.to_compile_error());
  // println!("{}", &toks);
  toks.into()
}

fn conf_tree(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let fields = if let syn::Data::Struct(ref data) = ast.data {
    &data.fields
  } else {
    return Err(syn::Error::new(ast.span(), "Expected struct"));
  };

  let field_names = fields
    .iter()
    .map(|f| {
      f.ident
        .as_ref()
        .ok_or(syn::Error::new(f.span(), "Expected field name"))
    })
    .collect::<syn::Result<Vec<_>>>()?;

  let field_strs = field_names
    .iter()
    .map(|ident| ident.to_string())
    .collect::<Vec<_>>();

  let opt_comma = if fields.is_empty() {
    quote! {}
  } else {
    quote! {,}
  };

  Ok(quote! {
    impl #impl_generics crate::config::ConfTree for #name #ty_generics #where_clause {
      fn get_child(&self, name: &str) -> Result<&dyn crate::config::ConfOpt, String> {
        match name {
          #(#field_strs => Ok(&self.#field_names)),* #opt_comma
          _ => Err(format!("unknown option {}", name)),
        }
      }
      fn get_child_mut(&mut self, name: &str) -> Result<&mut dyn crate::config::ConfOpt, String> {
        match name {
          #(#field_strs => Ok(&mut self.#field_names)),* #opt_comma
          _ => Err(format!("unknown option {}", name)),
        }
      }
    }
  })
}
