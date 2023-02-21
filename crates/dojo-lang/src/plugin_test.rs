use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::diagnostics::get_diagnostics_as_string;
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_defs::plugin::{MacroPlugin, PluginGeneratedFile, PluginResult};
use cairo_lang_formatter::format_string;
use cairo_lang_parser::db::ParserGroup;
use cairo_lang_semantic::test_utils::setup_test_module;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_test_utils::parse_test_file::TestFileRunner;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use dojo_project::{ProjectConfig, WorldConfig};

use crate::db::DojoRootDatabaseBuilderEx;
use crate::plugin::DojoPlugin;

struct ExpandContractTestRunner {
    db: RootDatabase,
}

impl Default for ExpandContractTestRunner {
    fn default() -> Self {
        Self {
            db: RootDatabase::builder().with_dojo_config(ProjectConfig::default()).build().unwrap(),
        }
    }
}
impl TestFileRunner for ExpandContractTestRunner {
    fn run(&mut self, inputs: &OrderedHashMap<String, String>) -> OrderedHashMap<String, String> {
        let (test_module, _semantic_diagnostics) =
            setup_test_module(&mut self.db, inputs["cairo_code"].as_str()).split();

        let file_id = self.db.module_main_file(test_module.module_id).unwrap();
        let syntax_file = self.db.file_syntax(file_id).unwrap();

        let plugin = DojoPlugin { world_config: WorldConfig::default() };
        let mut generated_items: Vec<String> = Vec::new();

        for item in syntax_file.items(&self.db).elements(&self.db).into_iter() {
            let PluginResult { code, diagnostics: _, remove_original_item } =
                plugin.generate_code(&self.db, item.clone());

            let content = match code {
                Some(PluginGeneratedFile { content, .. }) => content,
                None => continue,
            };
            if !remove_original_item {
                generated_items
                    .push(format_string(&self.db, item.as_syntax_node().get_text(&self.db)));
            }
            generated_items.push(format_string(&self.db, content));
        }

        OrderedHashMap::from([
            ("generated_cairo_code".into(), generated_items.join("\n")),
            ("expected_diagnostics".into(), get_diagnostics_as_string(&mut self.db)),
        ])
    }
}

cairo_lang_test_utils::test_file_test_with_runner!(
    expand_contract,
    "src/plugin_test_data",
    {
        component: "component",
        system: "system",
    },
    ExpandContractTestRunner
);