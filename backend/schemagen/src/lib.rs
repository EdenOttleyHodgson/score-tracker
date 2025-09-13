use std::{error::Error, fs, path::PathBuf};

use backend_lib::{
    connection::message::{ClientMessage, ServerMessage},
    state::room::{
        MemberState,
        pot::Pot,
        wager::{Wager, WagerOutcome},
    },
};
use schemars::{Schema, schema_for};

pub fn gen_schema() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../type_schemas");
    if let Err(e) = fs::DirBuilder::new().create(&out_dir) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e.into());
        };
    };
    let schemas = [
        (schema_for!(ServerMessage), "server_message"),
        (schema_for!(ClientMessage), "client_message"),
        (schema_for!(Pot), "pot"),
        (schema_for!(Wager), "wager"),
        (schema_for!(WagerOutcome), "wager_outcome"),
        (schema_for!(MemberState), "member_state"),
    ];
    for (schema, name) in schemas.iter() {
        write_schema(&out_dir, name, schema)?
    }

    Ok(())
}
fn write_schema(dir: &std::path::Path, name: &str, schema: &Schema) -> std::io::Result<()> {
    let output = serde_json::to_string_pretty(schema).unwrap();
    let output_path = dir.join(format!("{}.json", name));
    std::fs::write(output_path, output)
}
