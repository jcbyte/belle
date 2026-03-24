use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Hint, attributes(hint))]
pub fn derive_hint(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    // Only allow macro on enum variants
    let Data::Enum(data) = input.data else {
        return quote!(compile_error!("`Hint` is only allowed for enum types");).into();
    };

    let arms = data.variants.iter().map(|v| {
        let variant_ident = &v.ident;

        let mut is_transparent = false;
        let mut static_msg: Option<String> = None;

        // Parse attributes given
        for attr in &v.attrs {
            // Ensure this is our attribute
            if attr.path().is_ident("hint") {
                if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                    // If there is an inner identifier, match it
                    if ident == "transparent" {
                        // If the hint identifier is "transparent": #[hint(transparent)] then mark this as transparent
                        is_transparent = true;
                    }
                } else if let Ok(literal) = attr.parse_args::<syn::LitStr>() {
                    // If the hint has a string literal then it must contain a massage: #[hint("A hint")]
                    static_msg = Some(literal.value())
                } else {
                    return quote! {
                        compile_error!("`#[hint()]` must contain either `transparent` or a string literal");
                    };
                }
            }
        }

        // Generate match arm
        if is_transparent {
            // If transparent then get the hint from the inner object
            if let Fields::Unnamed(fields) = &v.fields
                && fields.unnamed.len() == 1
            {
                quote! {
                  #ident::#variant_ident(inner) => inner.get_hint()
                }
            } else {
                // Complain if there is multiple fields or incorrect type
                quote!(compile_error!("`#[hint(transparent)]` is only allowed on `Unnamed` variants with 1 field");)
            }
        } else {
            match &v.fields {
                Fields::Unit => {
                    // For unit fields just return the string reference
                    let msg = static_msg
                        .map(|m| quote!(Some(std::borrow::Cow::Borrowed(#m))))
                        .unwrap_or(quote!(None));
                    quote! {
                      #ident::#variant_ident => #msg
                    }
                }
                Fields::Unnamed(fields) => {
                    // For unnamed fields map each field to `arg_` and pass this through format
                    let field_idents: Vec<_> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format_ident!("arg{}", i))
                        .collect();

                    let msg = static_msg
                        .map(|m| quote!(Some(std::borrow::Cow::Owned(format!(#m)))))
                        .unwrap_or(quote!(None));
                    quote! {
                      #ident::#variant_ident( #(#field_idents),* ) => #msg
                    }
                }
                Fields::Named(fields) => {
                    // For named fields pass these directly though format
                    let field_idents = fields.named.iter().map(|f| &f.ident);
                    let msg = static_msg
                        .map(|m| quote!(Some(std::borrow::Cow::Owned(format!(#m)))))
                        .unwrap_or(quote!(None));
                    quote! {
                      #ident::#variant_ident { #(#field_idents),* } => #msg
                    }
                }
            }
        }
    });

    TokenStream::from(quote! {
      impl ::hinted::Hint for #ident {
        fn get_hint(&self) -> Option<std::borrow::Cow<'static, str>> {
          // Ignore these warnings as match arms will collect every value from enum types
          #[allow(unused_variables, unused_assignments)]
          match self {
            #(#arms),*
          }
        }
      }
    })
}
