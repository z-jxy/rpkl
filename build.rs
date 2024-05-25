// mod pkl_build {
//     use std::path::PathBuf;
//
//     use pkl_rs_build::api::evaluator;
//
//     pub fn configure() -> Result<(), Box<dyn std::error::Error>> {
//         Ok(())
//     }
//
//     pub fn compile(paths: &[&str], include: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
//         let mut evaluator = evaluator::Evaluator::new()?;
//         for path in paths.iter() {
//             let path = PathBuf::from(path)
//                 .canonicalize()
//                 .expect("failed to canonicalize path");
//             let pkl_mod = evaluator.evaluate_module(path.to_str().unwrap())?;
//             pkl_mod.emit_rust()?;
//         }
//         println!("[*] codegen success");
//         Ok(())
//     }
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // pkl_build::configure()?;
    // pkl_build::compile(&["./your-config.pkl"], &["./"])?;
    Ok(())
}
