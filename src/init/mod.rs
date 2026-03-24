use anyhow::{Context, Result};
use inquire;
use itertools::Itertools;
use minijinja;
use regex::Regex;
use serde;
use serde_yaml_ng::Value;
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, error, info, trace, warn};

use crate::configparser::config::{self, ProfileConfig, S3Config};
use crate::utils::render_strict;

pub mod example_values;
pub mod templates;

pub fn render_config(interactive: bool, placeholders: bool, blank: bool) -> Result<String> {
    let options = if blank {
        blank_init()
    } else if placeholders {
        placeholder_init()
    } else {
        // default to interactive if no flags given
        interactive_init()?
    };

    let configuration = templatize_init(&options).context("could not render template")?;

    // Note about external-dns
    warn!("Note: external-dns configuration settings will need to be provided in rcds.yaml after file creation, under the `profiles.<name>.dns` key.");
    warn!("Reference: https://kubernetes-sigs.github.io/external-dns/latest/charts/external-dns/");

    Ok(configuration)
}

fn interactive_init() -> inquire::error::InquireResult<config::RcdsConfig> {
    println!("For all prompts below, press Enter to leave blank.");
    println!("All fields that can be set in rcds.yaml can also be set via environment variables.");
    println!("See the docs for more info on each field: https://beavercds.info/reference/rcds-yaml-reference.html");

    let class_names; // set during `points` prompt later

    // FORMATTING NOTE: Some of these help messages cause rustfmt to silently
    // fail to format this struct definition. Commenting out the marked
    // help_message lines temporarily will let the formatting work.
    //
    // Ref:
    // - https://github.com/rust-lang/rustfmt/issues/6687
    // - https://github.com/rust-lang/rustfmt/issues/3863

    let options = config::RcdsConfig {
        //TODO: what flavor of regex is being validated and accepted
        flag_regex: inquire::Text::new("Flag regex:")
            .with_help_message("This regex will be used to validate the individual flags of your challenges later.") // too long to format
            .with_placeholder(example_values::FLAG_REGEX)
            .prompt()?,

        registry: config::Registry {
            tag_format: inquire::Text::new("Container image/tag format:")
                .with_help_message("Template to use for built container images. This default works with most registries.") // too long to format
                .with_placeholder(&config::default_tag_format())
                .prompt()?,
            domain: inquire::Text::new("Container registry:")
                .with_help_message("Registry domain and repository name of the container registry for hosted challenge images.") // too long to format
                .with_placeholder(example_values::REGISTRY_DOMAIN)
                .prompt()?,
            build: config::UserPass {
                user: inquire::Text::new("Container registry 'build' user:")
                    .with_help_message("The username that will be used to push built containers.")
                    .with_placeholder(example_values::REGISTRY_BUILD_USER)
                    .prompt()?,
                // TODO: do we actually want to be in charge of these credentials vs expecting the local building utility already be logged in?
                pass: inquire::Password::new("Container registry 'build' password:")
                    .with_help_message("The password to the 'build' user account.") // TODO: could this support username:pat too?
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .with_custom_confirmation_message("Enter again:")
                    .prompt()?,
            },
            cluster: config::UserPass {
                user: inquire::Text::new("Container registry 'cluster' user:")
                    .with_help_message(
                        "The username that the cluster will use to pull locally-built containers.",
                    )
                    .with_placeholder(example_values::REGISTRY_CLUSTER_USER)
                    .prompt()?,
                pass: inquire::Password::new("Container registry 'cluster' password:")
                    .with_help_message("The password to the 'cluster' user account.")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .with_custom_confirmation_message("Enter again:")
                    .prompt()?,
            },
        },

        point_classes: {
            println!("You can define several challenge point classes below.");
            let mut again = inquire::Confirm::new("Do you want to provide a point class?")
                .with_default(false)
                .prompt()?;

            println!("Challenge points are dynamic. For a static challenge, simply set minimum and maximum points to the same value.");
            let mut points = vec![];
            while again {
                let points_obj = config::PointClass {
                    name: inquire::Text::new("Point class:")
                        .with_validator(inquire::required!("Please provide a name."))
                        .with_help_message("The name of the point class.")
                        .with_placeholder(example_values::POINTS_EASY_CLASS)
                        .prompt()?,
                    min: inquire::CustomType::<i64>::new("Minimum points:")
                        .with_error_message("Please type a valid number.") // default parser calls std::u64::from_str
                        .with_help_message("The minimum number of points that challenges within this class are worth.") // too long to format
                        .with_default(example_values::POINTS_EASY_MIN)
                        .prompt()?,
                    max: inquire::CustomType::<i64>::new("Maximum points:")
                        .with_error_message("Please type a valid number.") // default parser calls std::u64::from_str
                        .with_help_message("The maximum number of points that challenges within this class are worth.") // too long to format
                        .with_default(example_values::POINTS_EASY_MAX)
                        .prompt()?,
                };
                points.push(points_obj);

                again = inquire::Confirm::new("Do you want to provide another point class?")
                    .with_default(false)
                    .prompt()?;
            }
            // save owned copy of difficulty category names for use below
            class_names = points.iter().map(|p| p.name.clone()).collect_vec();
            points
        },
        defaults: config::Defaults {
            point_class: {
                if class_names.is_empty() {
                    String::new()
                } else {
                    inquire::Select::new(
                        "Please choose the default point class:",
                        class_names,
                    )
                    .prompt()?
                }
            },

            resources: config::Resource {
                cpu: inquire::CustomType::<i64>::new("Default CPU limit:")
                    .with_help_message("The default limit of CPU resources per challenge pod.\nhttps://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#resource-units-in-kubernetes") // too long to format
                    .with_placeholder(&example_values::DEFAULTS_RESOURCES_CPU.to_string())
                    .with_default(example_values::DEFAULTS_RESOURCES_CPU)
                    .prompt()?,

                memory: inquire::Text::new("Default memory limit:")
                    .with_help_message("The default limit of CPU resources per challenge pod.\nhttps://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#resource-units-in-kubernetes") // too long to format
                    .with_placeholder(example_values::DEFAULTS_RESOURCES_MEMORY)
                    .with_default(example_values::DEFAULTS_RESOURCES_MEMORY)
                    .prompt()?,
            },
        },

        profiles: {
            println!("You can define several environment profiles below.");

            let mut again = inquire::Confirm::new("Do you want to provide a Profile?")
                .with_default(false)
                .prompt()?;
            let mut profiles = HashMap::new();

            while again {
                let name = inquire::Text::new("Profile name:")
                    .with_help_message("The name of the deployment Profile. One Profile named \"default\" is recommended. You can add additional profiles.") // too long to format
                    .with_placeholder(example_values::PROFILES_PROFILE_NAME)
                    .prompt()?;

                let prof = config::ProfileConfig {
                    frontend_url: inquire::Text::new("Frontend URL:")
                        .with_help_message("The URL of the RNG scoreboard.")
                        .with_placeholder(example_values::PROFILES_FRONTEND_URL)
                        .prompt()?,
                    frontend_token: inquire::Text::new("Frontend token:")
                        .with_help_message("The token to authenticate into the RNG scoreboard.")
                        .with_placeholder(example_values::PROFILES_FRONTEND_TOKEN)
                        .prompt()?,
                    challenges_domain: inquire::Text::new("Challenges domain:")
                        .with_help_message("Domain that challenges are hosted under.")
                        .with_placeholder(example_values::PROFILES_CHALLENGES_DOMAIN)
                        .prompt()?,
                    kubecontext: inquire::Text::new("Kubecontext name:")
                        .with_help_message(
                            "The name of the context that kubectl uses to connect to the cluster.",
                        )
                        .with_placeholder(example_values::PROFILES_KUBECONTEXT)
                        .prompt()?,
                    s3: config::S3Config {
                        bucket_name: inquire::Text::new("S3 bucket name:")
                            .with_help_message("Challenge artifacts and static files will be hosted on S3. The name of the S3 bucket.") // too long to format
                            .with_placeholder(example_values::PROFILES_S3_BUCKET_NAME)
                            .prompt()?,
                        endpoint: inquire::Text::new("S3 endpoint:")
                            .with_help_message("The endpoint of the S3 bucket server.")
                            .with_placeholder(example_values::PROFILES_S3_ENDPOINT)
                            .prompt()?,
                        region: inquire::Text::new("S3 region:")
                            .with_help_message("The region where the S3 bucket is hosted.")
                            .with_placeholder(example_values::PROFILES_S3_REGION)
                            .prompt()?,
                        access_key: inquire::Text::new("S3 access key:")
                            .with_help_message("The public access key to the S3 bucket.")
                            .with_placeholder(example_values::PROFILES_S3_ACCESSKEY)
                            .prompt()?,
                        secret_key: inquire::Text::new("S3 secret key:")
                            .with_help_message("The secret acess key to the S3 bucket.")
                            .with_placeholder(example_values::PROFILES_S3_SECRETACCESSKEY)
                            .prompt()?,
                    },
                    kubeconfig: None,
                    dns: Default::default(), // explicitly leave this blank, user needs to set it
                };

                profiles.insert(name, prof);

                again = inquire::Confirm::new("Do you want to provide another Profile?")
                    .with_default(false)
                    .prompt()?;
            }
            profiles
        },

        deploy: HashMap::new(), // user is init'ing a blank repo, no challenges yet!
    };

    Ok(options)
}

fn blank_init() -> config::RcdsConfig {
    trace!("building blank config");

    // struct does not implement Default on purpose, manually fill out as blank
    config::RcdsConfig {
        flag_regex: "".to_string(),
        registry: config::Registry {
            domain: "".to_string(),
            tag_format: String::new(),
            build: config::UserPass {
                user: "".to_string(),
                pass: "".to_string(),
            },
            cluster: config::UserPass {
                user: "".to_string(),
                pass: "".to_string(),
            },
        },
        defaults: config::Defaults {
            point_class: "".to_string(),
            resources: config::Resource {
                cpu: 0,
                memory: "".to_string(),
            },
        },
        point_classes: vec![],
        deploy: HashMap::from([]),
        profiles: HashMap::from([]),
    }
}

fn placeholder_init() -> config::RcdsConfig {
    trace!("building placeholder values config");

    config::RcdsConfig {
        flag_regex: example_values::FLAG_REGEX.to_string(),
        registry: config::Registry {
            domain: example_values::REGISTRY_DOMAIN.to_string(),
            tag_format: config::default_tag_format(),
            build: config::UserPass {
                user: example_values::REGISTRY_BUILD_USER.to_string(),
                pass: example_values::REGISTRY_BUILD_PASS.to_string(),
            },
            cluster: config::UserPass {
                user: example_values::REGISTRY_CLUSTER_USER.to_string(),
                pass: example_values::REGISTRY_CLUSTER_PASS.to_string(),
            },
        },
        defaults: config::Defaults {
            point_class: example_values::DEFAULTS_CLASS.to_string(),
            resources: config::Resource {
                cpu: example_values::DEFAULTS_RESOURCES_CPU,
                memory: example_values::DEFAULTS_RESOURCES_MEMORY.to_string(),
            },
        },
        point_classes: vec![
            config::PointClass {
                name: example_values::POINTS_EASY_CLASS.to_string(),
                min: example_values::POINTS_EASY_MIN,
                max: example_values::POINTS_EASY_MAX,
            },
            config::PointClass {
                name: example_values::POINTS_HARD_CLASS.to_string(),
                min: example_values::POINTS_HARD_MIN,
                max: example_values::POINTS_HARD_MAX,
            },
        ],

        deploy: HashMap::from([(
            example_values::PROFILES_PROFILE_NAME.to_string(),
            config::ProfileDeploy {
                challenges: HashMap::new(),
            },
        )]),

        profiles: HashMap::from([(
            example_values::PROFILES_PROFILE_NAME.to_string(),
            ProfileConfig {
                frontend_url: example_values::PROFILES_FRONTEND_URL.to_string(),
                frontend_token: example_values::PROFILES_FRONTEND_TOKEN.to_string(),
                challenges_domain: example_values::PROFILES_CHALLENGES_DOMAIN.to_string(),
                kubeconfig: None,
                kubecontext: example_values::PROFILES_KUBECONTEXT.to_string(),
                s3: S3Config {
                    bucket_name: example_values::PROFILES_S3_BUCKET_NAME.to_string(),
                    endpoint: example_values::PROFILES_S3_ENDPOINT.to_string(),
                    region: example_values::PROFILES_S3_REGION.to_string(),
                    access_key: example_values::PROFILES_S3_ACCESSKEY.to_string(),
                    secret_key: example_values::PROFILES_S3_SECRETACCESSKEY.to_string(),
                },
                dns: Value::Null,
            },
        )]),
    }
}

fn templatize_init(options: &config::RcdsConfig) -> Result<String> {
    debug!("rendering template with {options:?}");
    render_strict(
        templates::RCDS,
        minijinja::context! {.. minijinja::Value::from_serialize(options)},
    )
}
