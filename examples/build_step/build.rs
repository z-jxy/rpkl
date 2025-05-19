use rpkl::EvaluatorOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpkl::build_script::configure()
        .as_enum("Example.mode", &["Dev", "Production"])
        .opaque("Example.mapping")
        .evaluator_options(EvaluatorOptions::default())
        // .output("generated/mod.rs")
        .codegen(&[
            "../../tests/pkl/example.pkl",
            "../../tests/pkl/database.pkl",
        ])
}
