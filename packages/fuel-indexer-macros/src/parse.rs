use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

mod kw {
    syn::custom_keyword!(manifest);
}

pub(crate) struct IndexerConfig {
    pub(crate) manifest: String,
}

struct ConfigBuilder {
    manifest: Option<Manifest>,
}

impl ConfigBuilder {
    fn new() -> ConfigBuilder {
        ConfigBuilder { manifest: None }
    }

    fn set_manifest(&mut self, manifest: Manifest) {
        self.manifest = Some(manifest)
    }

    fn build(self) -> IndexerConfig {
        let ConfigBuilder { manifest } = self;

        IndexerConfig {
            manifest: manifest
                .expect("`manifest` specification is required in indexer definition.")
                .name
                .value(),
        }
    }
}

impl Parse for IndexerConfig {
    fn parse(input: ParseStream) -> syn::Result<IndexerConfig> {
        let mut config = ConfigBuilder::new();

        let items = Punctuated::<ConfigItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ConfigItem::Manifest(s) => config.set_manifest(s),
            }
        }

        Ok(config.build())
    }
}

enum ConfigItem {
    Manifest(Manifest),
}

impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> syn::Result<ConfigItem> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::manifest) {
            Ok(input.parse().map(ConfigItem::Manifest)?)
        } else {
            Err(lookahead.error())
        }
    }
}

struct Manifest {
    name: LitStr,
}

impl Parse for Manifest {
    fn parse(input: ParseStream) -> syn::Result<Manifest> {
        let _: kw::manifest = input.parse()?;
        let _: Token![=] = input.parse()?;
        let name = input.parse()?;

        Ok(Manifest { name })
    }
}
