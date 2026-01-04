use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive_oxidx_component(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Find fields
    let mut children = Vec::new();
    let mut bounds_field = None;
    let mut id_field = None;

    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let mut is_child = false;
                let mut is_bounds = false;
                let mut is_id = false;

                // Check attributes
                for attr in &field.attrs {
                    if attr.path().is_ident("oxidx") {
                        if let Err(err) = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("child") {
                                is_child = true;
                            } else if meta.path.is_ident("bounds") {
                                is_bounds = true;
                            } else if meta.path.is_ident("id") {
                                is_id = true;
                            }
                            Ok(())
                        }) {
                            return Err(err);
                        }
                    }
                }

                // Heuristics if no explicit attribute
                if field_name == "bounds" && !is_child && !is_id {
                    is_bounds = true;
                }
                if field_name == "id" && !is_child && !is_bounds {
                    is_id = true;
                }

                if is_child {
                    children.push(field_name);
                }

                // Only set if not already set (attributes take precedence)
                if is_bounds {
                    bounds_field = Some(field_name);
                }
                if is_id {
                    id_field = Some(field_name);
                }
            }
        }
    }

    let bounds_impl = if let Some(bounds) = bounds_field {
        quote! {
            fn bounds(&self) -> oxidx_core::primitives::Rect {
                self.#bounds
            }
            fn set_position(&mut self, x: f32, y: f32) {
                self.#bounds.x = x;
                self.#bounds.y = y;
            }
            fn set_size(&mut self, width: f32, height: f32) {
                self.#bounds.width = width;
                self.#bounds.height = height;
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            input,
            "Could not find a 'bounds' field. Please add a field named 'bounds' or mark one with #[oxidx(bounds)]."
        ));
    };

    let id_impl = if let Some(id) = id_field {
        quote! {
            fn id(&self) -> &str {
                &self.#id
            }
        }
    } else {
        quote! {
            fn id(&self) -> &str { "" }
        }
    };

    let update_impl = quote! {
        fn update(&mut self, dt: f32) {
            #(self.#children.update(dt);)*
        }
    };

    let layout_impl = quote! {
        fn layout(&mut self, available: oxidx_core::primitives::Rect) -> oxidx_core::Vec2 {
            self.layout_content(available)
        }
    };

    // Render: Background -> Children -> Foreground
    let render_impl = quote! {
        fn render(&self, renderer: &mut oxidx_core::renderer::Renderer) {
            self.render_background(renderer);
            #(self.#children.render(renderer);)*
            self.render_foreground(renderer);
        }
    };

    // Events: Logic -> Children
    let event_impl = quote! {
        fn on_event(&mut self, event: &oxidx_core::events::OxidXEvent, ctx: &mut oxidx_core::OxidXContext) -> bool {
            if self.handle_event(event, ctx) {
                return true;
            }
            #(
                if self.#children.on_event(event, ctx) {
                    return true;
                }
            )*
            false
        }
    };

    // Keyboard: Children -> Logic
    let keyboard_impl = quote! {
        fn on_keyboard_input(&mut self, event: &oxidx_core::events::OxidXEvent, ctx: &mut oxidx_core::OxidXContext) {
            #(self.#children.on_keyboard_input(event, ctx);)*
            self.handle_keyboard(event, ctx);
        }
    };

    let focusable_impl = quote! {
         fn is_focusable(&self) -> bool { false }
    };

    let child_count_impl = {
        let count = children.len();
        quote! {
            fn child_count(&self) -> usize { #count }
        }
    };

    Ok(quote! {
        impl oxidx_core::component::OxidXComponent for #struct_name {
            #bounds_impl
            #id_impl
            #update_impl
            #layout_impl
            #render_impl
            #event_impl
            #keyboard_impl
            #focusable_impl
            #child_count_impl
        }
    })
}
