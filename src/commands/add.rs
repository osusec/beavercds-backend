use std::fs;
use simplelog::error;
use std::process::exit;
use inquire;

use crate::add;

pub fn run (_name: &str, _category: &str, _interactive: &bool, _blank: &bool)
{
    let options: add::AddVars;

    let category_attr = fs::metadata(_category);
    if (&category_attr).is_ok() && (&category_attr).unwrap().is_file() {
        error!("Category `{_category}` is already a file: cannot make directory.");
        exit(1);
    }

    if *_interactive {
        // if the path doesn't exist then it is a new category
        if (&category_attr).is_err() && add::i_category_check(_category) {
            create_category(_category);
        }

        options = match add::interactive_add(_name) {
            Ok(t) => t,
            Err(e) => {
                error!("Error in add: {e}");
                exit(1);
            }
        };
    } else if *_blank {
        if (&category_attr).is_err() {
            create_category(_category);
        }

        options = add::blank_add(_name);
    } else {
        if (&category_attr).is_err() {
            create_category(_category);
        }

        options = add::example_add(_name);
    }

    // let config_path = format!("{}/{}/challenge.yaml", _category, _name);
    // let configuration = add::templatize_challenge(options);
    // let mut f = match File::create(config_path) {
    //      Ok(t) => t,
    //      Err(e) => {
    //          error!("Error in add: {e}");
    //          exit(1);
    //      }
    // };
    // match f.write_all(configuration.as_bytes()) {
    //     Ok(_) => (),
    //     Err(e) => {
    //         error!("Error in add: {e}");
    //         exit(1);
    //     }
    // }
}

pub fn create_category (_category: &str) {//-> Result<_, Error>{
    // TODO
}
