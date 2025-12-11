#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::init::{blank_init, example_init, interactive_init, templatize_init};

#[test]
/// Config template renders correctly with blank init
fn blank_init_rendering() {
    let blank = blank_init();

    let rendered = templatize_init(&blank);
    assert!(rendered.is_ok(), "blank template failed to render");
    assert_eq!(
        rendered.unwrap(),
        r#"# Used to check that all challenges' flags are in the correct format,
# and by the scoreboard frontend as a first check for invalid submissions.
flag_regex: ""

# Registry configuration for challenge images.
registry:
  domain: ""
  # This is the default tag format; it will create a separate image for each
  # challenge pod. Most container registries (Docker, GHCR, Gitlab, Quay, ...)
  # are fine with this. If you are using a container registry that requires
  # every image within the repository to be created ahead-of-time (AWS ECR)
  # before it can be pushed, you can change this to use tags for each separate
  # challenge within one image in the registry.
  tag_format: ""
  # Build-time credentials used to push images during `beavercds deploy`.
  build:
    user: ""
    pass: ""
  # Used by the cluster to pull the built images.
  cluster:
    user: ""
    pass: ""

# Default difficulty class and resource requests used for challenges that did
# not set their own.
defaults:
  difficulty: ""
  resources: { cpu: 0, memory: "" }

# The list of different difficulties that challenges can be assigned, and how
# many points challenges of that difficulty class should be worth. All
# challenges use dynamic scoring; for static points set both min and max to the
# same value.
points:
  []

# Control what challenges are deployed in each environment profile.
deploy:
  {}

# Separate environment profiles to allow for multiple independent deployments
# of challenges, e.g. staging and production to test challenges internally
# before going live for all users.
profiles:
  {}
"#
    );
}

#[test]
/// Config template renders correctly with example values
fn example_init_rendering() {
    let examples = example_init();

    let rendered = templatize_init(&examples);
    assert!(rendered.is_ok(), "example template failed to render");
    assert_eq!(
        rendered.unwrap(),
        r#"# Used to check that all challenges' flags are in the correct format,
# and by the scoreboard frontend as a first check for invalid submissions.
flag_regex: "ctf{.*}"

# Registry configuration for challenge images.
registry:
  domain: "ghcr.io/youraccount"
  # This is the default tag format; it will create a separate image for each
  # challenge pod. Most container registries (Docker, GHCR, Gitlab, Quay, ...)
  # are fine with this. If you are using a container registry that requires
  # every image within the repository to be created ahead-of-time (AWS ECR)
  # before it can be pushed, you can change this to use tags for each separate
  # challenge within one image in the registry.
  tag_format: "{{domain}}/{{challenge}}-{{container}}:{{profile}}"
  # Build-time credentials used to push images during `beavercds deploy`.
  build:
    user: "build_user"
    pass: "notrealcreds"
  # Used by the cluster to pull the built images.
  cluster:
    user: "cluster_user"
    pass: "alsofake"

# Default difficulty class and resource requests used for challenges that did
# not set their own.
defaults:
  difficulty: "easy"
  resources: { cpu: 1, memory: "500M" }

# The list of different difficulties that challenges can be assigned, and how
# many points challenges of that difficulty class should be worth. All
# challenges use dynamic scoring; for static points set both min and max to the
# same value.
points:
  - difficulty: "easy"
    min: 200
    max: 500
  - difficulty: "hard"
    min: 300
    max: 600

# Control what challenges are deployed in each environment profile.
deploy:
  default: {}

# Separate environment profiles to allow for multiple independent deployments
# of challenges, e.g. staging and production to test challenges internally
# before going live for all users.
profiles:
  default:
    # Used to push challenge information into the frontend/scoreboard.
    frontend_url: "https://ctf.coolguy.invalid"
    frontend_token: "secretsecretsecret"
    # Root domain to expose all challenges under.
    challenges_domain: "chals.coolguy.invalid"
    # Kubernetes kubeconfig and context name of cluster for this profile.
    kubecontext: "ctf-cluster"
    # Credentials for the public challenge file asset bucket.
    s3:
      bucket_name: "ctf-bucket"
      endpoint: "s3.coolguy.invalid"
      region: "us-west-2"
      access_key: "accesskey"
      secret_key: "secretkey"
    # Config for the environment's external-dns deployment.
    dns:
      # Place external-dns configuration options here;
      # this yaml will be passed directly to external-dns without modification
      # Reference: https://github.com/bitnami/charts/tree/main/bitnami/external-dns
"#
    );
}

#[test]
/// Config template renders correctly with faked interactive input
fn interactive_init_rendering() {
    println!("TODO: interactive testing not implemented");

    // TODO: how to test the prompts? inquire does not offer a test mode, see
    // https://github.com/mikaelmello/inquire/issues/71.
    // Possible solution: shell out and test output via https://github.com/rust-cli/rexpect

    // let interactive = interactive_init();
    // assert!(interactive.is_ok(), "interactive prompts failed in testing");

    // let rendered = templatize_init(&interactive.unwrap());
    // assert!(rendered.is_ok(), "example template failed to render");
    // assert_eq!(rendered.unwrap(), "");
}
