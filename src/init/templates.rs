// Embed template file into binary

pub static RCDS: &str = include_str!("../asset_files/init_template/rcds.yaml.j2");
pub static SCRIPTS_NEW_CHAL: &str =
    include_str!("../asset_files/init_template/scripts/new-chal.py");
pub static README: &str = include_str!("../asset_files/init_template/README.md");
