use inquire::{self, validator::StringValidator};

pub mod templates;
pub mod example_values;


pub struct AddVars {
    pub name: String,
    pub author: String,
    pub description: String,
    pub difficulty: String, // matching an InitVars.points[i].difficulty
    pub flag: FlagStyle,
    pub provide: Vec<Provides>,
    pub pods: Vec<Pod>
}

pub enum FlagStyle {
    FlagFile(String),
    FlagText(String),
    FlagRegex(String)
}

pub enum Provides {
    FileProvide(String),
    PodProvide(PodProvides)
}

pub struct PodProvides {
    pub from_pod: String,
    pub as_file: String,
    pub include_files: Vec<String>
}

pub struct Pod {
    pub name: String,
    pub build_image: PodSrc,
    pub env: String, // TODO: another block
    pub resources: String, // TODO: optional
    pub replicas: String, // TODO: default 1?
    pub ports: Vec<Port>,
}

pub enum PodSrc {
    PodBuild(String), // one-liner / "context"
    PodImage(String)
}

pub struct Port {
    pub internal: String,
    pub expose: ExposePort
}

pub enum ExposePort {
    ExposeTCP(String),
    ExposeHTTP(String)
}


pub fn interactive_add(_name: &str) {
    //
}

pub fn blank_add(_name: &str) {
    //
}

pub fn example_add(_name: &str) -> AddVars {
    return AddVars {
        name: String::from(_name),
        author: String::from(example_values::CHAL_AUTHOR),
        description: String::from(example_values::CHAL_DESC),
        difficulty: {
            // TODO
            String::from("ass")
        },
        flag: FlagStyle::FlagFile(String::from(example_values::FILE_FLAG)),
        provide: vec![
            Provides::FileProvide(String::from(example_values::PROVIDE_STR)),
            Provides::PodProvide(PodProvides {
                from_pod: String::from(example_values::PROVIDE_POD_FROM),
                as_file: String::from(example_values::PROVIDE_POD_AS_FILE),
                include_files: vec![
                    String::from(example_values::PROVIDE_POD_INCLUDE1),
                    String::from(example_values::PROVIDE_POD_INCLUDE2)
                ]
            }),
        ],
        pods: vec![
            Pod {
                name: String::from(example_values::PODS_NAME),
                build_image: PodSrc::PodBuild(String::from(example_values::PODS_SRC_BUILD)),
                env: String::new(), // TODO
                resources: String::new(), // TODO
                replicas: String::from(example_values::PODS_REPLICAS),
                ports: {
                    vec![
                        Port {
                            internal: String::from(example_values::PODS_PORTS_INTERNAL_HTTP),
                            expose: ExposePort::ExposeHTTP(String::from(example_values::PODS_PORTS_INTERNAL_HTTP))
                        }
                    ]
                }
            }
        ]
    };
}

pub fn templatize_challenge(options: AddVars) {
    //
}

pub fn i_category_check (_category: &str) -> bool // TODO: needs testing
{
    return match inquire::Confirm::new("This category does not already exist. Do you want to create it?")
    .with_help_message(format!("This will create the directory `{_category}`.").as_str())
    .with_default(false)
    .prompt()
    {
        Ok(t) => t,
        Err(e) => false
    };
}
