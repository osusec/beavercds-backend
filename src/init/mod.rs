use anyhow::Result;
use inquire;
use minijinja;
use regex::Regex;
use serde;
use std::fmt;
use tracing::{debug, error, info, trace, warn};

use crate::utils::render_strict;

pub mod example_values;
pub mod templates;

#[derive(serde::Serialize, Default, Debug)]
pub struct InitVars {
    pub flag_regex: String,
    pub registry_domain: String,
    pub registry_build_user: String,
    pub registry_build_pass: String,
    pub registry_cluster_user: String,
    pub registry_cluster_pass: String,
    pub defaults_difficulty: String,
    pub defaults_resources_cpu: String,
    pub defaults_resources_memory: String,
    pub points: Vec<Points>,
    pub profiles: Vec<Profile>,
}

#[derive(Clone, serde::Serialize, Default, Debug)]
pub struct Points {
    pub difficulty: String,
    pub min: String,
    pub max: String,
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}  Points: {}-{})",
            self.difficulty, self.min, self.max
        )
    }
}

#[derive(serde::Serialize, Default, Debug)]
pub struct Profile {
    pub profile_name: String,
    pub frontend_url: String,
    pub frontend_token: String,
    pub challenges_domain: String,
    pub kubecontext: String,
    pub s3_bucket_name: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_accesskey: String,
    pub s3_secretaccesskey: String,
}

pub fn interactive_init() -> inquire::error::InquireResult<InitVars> {
    println!("For all prompts below, simply press Enter to leave blank.");
    println!("All fields that can be set in rcds.yaml can also be set via environment variables.");

    let points_ranks_reference: Vec<Points>;

    let options = InitVars {
        flag_regex: {
            //TODO: what flavor of regex is being validated and accepted
            inquire::Text::new("Flag regex:")
            .with_help_message("This regex will be used to validate the individual flags of your challenges later.")
            .with_placeholder(example_values::FLAG_REGEX)
            .prompt()?
        },

        registry_domain: {
            inquire::Text::new ("Container registry:")
            .with_help_message("Hosted challenges will be hosted in a container registry.The connection endpoint and the repository name.")
            .with_placeholder(example_values::REGISTRY_DOMAIN)
            .prompt()?
        },

        registry_build_user: {
            inquire::Text::new("Container registry 'build' user:")
                .with_help_message("The username that will be used to push built containers.")
                .with_placeholder(example_values::REGISTRY_BUILD_USER)
                .prompt()?
        },

        // TODO: do we actually want to be in charge of these credentials vs expecting the local building utility already be logged in?
        registry_build_pass: {
            inquire::Password::new("Container registry 'build' password:")
                .with_help_message("The password to the 'build' user account") // TODO: could this support username:pat too?
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .with_custom_confirmation_message("Enter again:")
                .prompt()?
        },

        registry_cluster_user: {
            inquire::Text::new("Container registry 'cluster' user:")
                .with_help_message(
                    "The username that the cluster will use to pull locally-built containers.",
                )
                .with_placeholder(example_values::REGISTRY_CLUSTER_USER)
                .prompt()?
        },

        registry_cluster_pass: {
            inquire::Password::new("Container registry 'cluster' password:")
                .with_help_message("The password to the 'cluster' user account")
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .with_custom_confirmation_message("Enter again:")
                .prompt()?
        },

        points: {
            println!("You can define several challenge difficulty classes below.");
            let mut again = inquire::Confirm::new("Do you want to provide a difficulty class?")
                .with_default(false)
                .prompt()?;
            println!("Challenge points are dynamic. For a static challenge, simply set minimum and maximum points to the same value.");
            let mut points_ranks: Vec<Points> = Vec::new();
            while again {
                let points_obj = Points {
                    difficulty: {
                        inquire::Text::new("Difficulty class:")
                            .with_validator(inquire::required!("Please provide a name."))
                            .with_help_message("The name of the difficulty class.")
                            .with_placeholder(example_values::POINTS_DIFFICULTY)
                            .prompt()?
                    },
                    min: {
                        inquire::CustomType::<u64>::new("Minimum points:")
                        .with_error_message("Please type a valid number.") // default parser calls std::u64::from_str
                        .with_help_message("The minimum number of points that challenges within this difficulty class are worth.")
                        .with_placeholder(example_values::POINTS_MIN)
                        .prompt()?
                        .to_string()
                    },
                    max: {
                        inquire::CustomType::<u64>::new("Maximum points:")
                        .with_error_message("Please type a valid number.") // default parser calls std::u64::from_str
                        .with_help_message("The maximum number of points that challenges within this difficulty class are worth.")
                        .with_placeholder(example_values::POINTS_MAX)
                        .prompt()?
                        .to_string()
                    },
                };
                points_ranks.push(points_obj);

                again = inquire::Confirm::new("Do you want to provide another difficulty class?")
                    .with_default(false)
                    .prompt()?;
            }
            points_ranks_reference = points_ranks.clone();
            points_ranks
        },

        defaults_difficulty: {
            if points_ranks_reference.is_empty() {
                String::new()
            } else {
                inquire::Select::new(
                    "Please choose the default difficulty class:",
                    points_ranks_reference,
                )
                .prompt()?
                .difficulty
            }
        },

        defaults_resources_cpu:   inquire::Text::new("Default CPU limit:")
            .with_help_message("The default limit of CPU resources per challenge pod.\nhttps://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#resource-units-in-kubernetes")
            .with_placeholder(example_values::DEFAULTS_RESOURCES_CPU)
            .with_default(example_values::DEFAULTS_RESOURCES_CPU)
            .prompt()?
        ,

        defaults_resources_memory: {
            inquire::Text::new("Default memory limit:")
            .with_help_message("The default limit of CPU resources per challenge pod.\nhttps://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#resource-units-in-kubernetes")
            .with_placeholder(example_values::DEFAULTS_RESOURCES_MEMORY)
            .with_default(example_values::DEFAULTS_RESOURCES_MEMORY)
            .prompt()?

        },

        profiles: {
            println!("You can define several environment profiles below.");

            let mut again = inquire::Confirm::new("Do you want to provide a Profile?")
                .with_default(false)
                .prompt()?;
            let mut profiles: Vec<Profile> = Vec::new();
            while again {
                let prof = Profile {
                    profile_name: {
                        inquire::Text::new("Profile name:")
                        .with_help_message("The name of the deployment Profile. One Profile named \"default\" is recommended. You can add additional profiles.")
                        .with_placeholder(example_values::PROFILES_PROFILE_NAME)
                        .prompt()?
                    },
                    frontend_url: {
                        inquire::Text::new("Frontend URL:")
                            .with_help_message("The URL of the RNG scoreboard.")
                            .with_placeholder(example_values::PROFILES_FRONTEND_URL)
                            .prompt()?
                    },
                    frontend_token: {
                        inquire::Text::new("Frontend token:")
                            .with_help_message("The token to authenticate into the RNG scoreboard.")
                            .with_placeholder(example_values::PROFILES_FRONTEND_TOKEN)
                            .prompt()?
                    },
                    challenges_domain: {
                        inquire::Text::new("Challenges domain:")
                            .with_help_message("Domain that challenges are hosted under.")
                            .with_placeholder(example_values::PROFILES_CHALLENGES_DOMAIN)
                            .prompt()?
                    },
                    kubecontext: {
                        inquire::Text::new("Kubecontext name:")
                        .with_help_message("The name of the context that kubectl uses to connect to the cluster.")
                        .with_placeholder(example_values::PROFILES_KUBECONTEXT)
                        .prompt()?
                    },
                    s3_bucket_name: {
                        inquire::Text::new("S3 bucket name:")
                        .with_help_message("Challenge artifacts and static files will be hosted on S3. The name of the S3 bucket.")
                        .with_placeholder(example_values::PROFILES_S3_BUCKET_NAME)
                        .prompt()?
                    },
                    s3_endpoint: {
                        inquire::Text::new("S3 endpoint:")
                            .with_help_message("The endpoint of the S3 bucket server.")
                            .with_placeholder(example_values::PROFILES_S3_ENDPOINT)
                            .prompt()?
                    },
                    s3_region: {
                        inquire::Text::new("S3 region:")
                            .with_help_message("The region where the S3 bucket is hosted.")
                            .with_placeholder(example_values::PROFILES_S3_REGION)
                            .prompt()?
                    },
                    s3_accesskey: {
                        inquire::Text::new("S3 access key:")
                            .with_help_message("The public access key to the S3 bucket.")
                            .with_placeholder(example_values::PROFILES_S3_ACCESSKEY)
                            .prompt()?
                    },
                    s3_secretaccesskey: {
                        inquire::Text::new("S3 secret key:")
                            .with_help_message("The secret acess key to the S3 bucket.")
                            .with_placeholder(example_values::PROFILES_S3_SECRETACCESSKEY)
                            .prompt()?
                    },
                };
                profiles.push(prof);

                again = inquire::Confirm::new("Do you want to provide another Profile?")
                    .with_default(false)
                    .prompt()?;
            }
            profiles
        },
    };

    Ok(options)
}

pub fn blank_init() -> InitVars {
    trace!("building blank config");
    InitVars::default()
}

pub fn example_init() -> InitVars {
    trace!("building example values config");
    InitVars {
        flag_regex: example_values::FLAG_REGEX.to_string(),
        registry_domain: example_values::REGISTRY_DOMAIN.to_string(),
        registry_build_user: example_values::REGISTRY_BUILD_USER.to_string(),
        registry_build_pass: example_values::REGISTRY_BUILD_PASS.to_string(),
        registry_cluster_user: example_values::REGISTRY_CLUSTER_USER.to_string(),
        registry_cluster_pass: example_values::REGISTRY_CLUSTER_USER.to_string(),
        defaults_difficulty: example_values::DEFAULTS_DIFFICULTY.to_string(),
        defaults_resources_cpu: example_values::DEFAULTS_RESOURCES_CPU.to_string(),
        defaults_resources_memory: example_values::DEFAULTS_RESOURCES_MEMORY.to_string(),
        points: vec![
            Points {
                difficulty: example_values::POINTS_DIFFICULTY.to_string(),
                min: example_values::POINTS_MIN.to_string(),
                max: example_values::POINTS_MAX.to_string(),
            },
            Points {
                difficulty: "2".to_string(),
                min: "1".to_string(),
                max: "1337".to_string(),
            },
        ],
        profiles: vec![Profile {
            profile_name: example_values::PROFILES_PROFILE_NAME.to_string(),
            frontend_url: example_values::PROFILES_FRONTEND_URL.to_string(),
            frontend_token: example_values::PROFILES_FRONTEND_TOKEN.to_string(),
            challenges_domain: example_values::PROFILES_CHALLENGES_DOMAIN.to_string(),
            kubecontext: example_values::PROFILES_KUBECONTEXT.to_string(),
            s3_bucket_name: example_values::PROFILES_S3_BUCKET_NAME.to_string(),
            s3_endpoint: example_values::PROFILES_S3_ENDPOINT.to_string(),
            s3_region: example_values::PROFILES_S3_REGION.to_string(),
            s3_accesskey: example_values::PROFILES_S3_ACCESSKEY.to_string(),
            s3_secretaccesskey: example_values::PROFILES_S3_SECRETACCESSKEY.to_string(),
        }],
    }
}

pub fn templatize_init(options: InitVars) -> Result<String> {
    debug!("rendering template with {options:?}");
    render_strict(templates::RCDS, minijinja::context! {options})
}
