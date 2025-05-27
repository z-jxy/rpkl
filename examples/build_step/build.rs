use rpkl::EvaluatorOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpkl::build_script::configure()
        .as_enum("Example.mode", &["Dev", "Production"])
        .opaque("Example.mapping")
        // derives Clone to the `Database` struct in the module `database`
        // ```
        // mod database {
        //     #[derive(Clone)]
        //     pub struct Database;
        // }
        // ```
        //
        // To target the outer struct generated for the module, use `Database` as the identifier.
        .type_attribute("database.Database", "#[derive(Clone)]")
        .evaluator_options(EvaluatorOptions::default())
        .codegen(&[
            "../../tests/pkl/example.pkl",
            "../../tests/pkl/database.pkl",
        ])
}
