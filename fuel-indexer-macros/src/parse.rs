use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

mod kw {
    syn::custom_keyword!(abi);
    syn::custom_keyword!(schema);
    syn::custom_keyword!(namespace);
    syn::custom_keyword!(native);
}

pub(crate) struct IndexerConfig {
    pub abi: String,
    pub namespace: String,
    pub schema: String,
    pub native: bool,
}

struct ConfigBuilder {
    abi: Option<Abi>,
    schema: Option<Schema>,
    namespace: Option<Namespace>,
    native: bool,
}

impl ConfigBuilder {
    fn new() -> ConfigBuilder {
        ConfigBuilder {
            abi: None,
            schema: None,
            namespace: None,
            native: false,
        }
    }

    fn set_abi(&mut self, abi: Abi) {
        self.abi = Some(abi);
    }

    fn set_namespace(&mut self, namespace: Namespace) {
        self.namespace = Some(namespace)
    }

    fn set_schema(&mut self, schema: Schema) {
        self.schema = Some(schema);
    }

    fn set_native(&mut self) {
        self.native = true;
    }

    fn build(self) -> IndexerConfig {
        let ConfigBuilder {
            abi,
            namespace,
            schema,
            native,
        } = self;

        if abi.is_none() {
            proc_macro_error::abort_call_site!(
                "abi specification is required in indexer definition!"
            )
        }

        if schema.is_none() {
            proc_macro_error::abort_call_site!(
                "schema specification is required in indexer definition!"
            )
        }

        if namespace.is_none() {
            proc_macro_error::abort_call_site!("namespace is required in indexer definition!")
        }

        let namespace = namespace.unwrap();
        let schema = schema.unwrap();
        let abi = abi.unwrap();

        IndexerConfig {
            abi: abi.name.value(),
            schema: schema.name.value(),
            namespace: namespace.name.value(),
            native,
        }
    }
}

impl Parse for IndexerConfig {
    fn parse(input: ParseStream) -> syn::Result<IndexerConfig> {
        let mut config = ConfigBuilder::new();

        let items = Punctuated::<ConfigItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ConfigItem::Abi(s) => config.set_abi(s),
                ConfigItem::Schema(s) => config.set_schema(s),
                ConfigItem::Namespace(s) => config.set_namespace(s),
                ConfigItem::Native => config.set_native(),
            }
        }

        Ok(config.build())
    }
}

enum ConfigItem {
    Abi(Abi),
    Schema(Schema),
    Namespace(Namespace),
    Native,
}

impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> syn::Result<ConfigItem> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::schema) {
            Ok(input.parse().map(ConfigItem::Schema)?)
        } else if lookahead.peek(kw::abi) {
            Ok(input.parse().map(ConfigItem::Abi)?)
        } else if lookahead.peek(kw::namespace) {
            Ok(input.parse().map(ConfigItem::Namespace)?)
        } else if lookahead.peek(kw::native) {
            let _: kw::native = input.parse()?;
            Ok(ConfigItem::Native)
        } else {
            Err(lookahead.error())
        }
    }
}

struct Abi {
    name: LitStr,
}

impl Parse for Abi {
    fn parse(input: ParseStream) -> syn::Result<Abi> {
        let _: kw::abi = input.parse()?;
        let _: Token![=] = input.parse()?;
        let name = input.parse()?;

        Ok(Abi { name })
    }
}

struct Namespace {
    name: LitStr,
}

impl Parse for Namespace {
    fn parse(input: ParseStream) -> syn::Result<Namespace> {
        let _: kw::namespace = input.parse()?;
        let _: Token![=] = input.parse()?;
        let name = input.parse()?;

        Ok(Namespace { name })
    }
}

struct Schema {
    name: LitStr,
}

impl Parse for Schema {
    fn parse(input: ParseStream) -> syn::Result<Schema> {
        let _: kw::schema = input.parse()?;
        let _: Token![=] = input.parse()?;
        let name = input.parse()?;

        Ok(Schema { name })
    }
}
