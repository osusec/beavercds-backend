// Example strings for rcds.yaml

pub static FLAG_REGEX: &str = "ctf{.*}";

pub static REGISTRY_DOMAIN: &str = "ghcr.io/youraccount";
pub static REGISTRY_BUILD_USER: &str = "build_user";
pub static REGISTRY_BUILD_PASS: &str = "notrealcreds";
pub static REGISTRY_CLUSTER_USER: &str = "cluster_user";
pub static REGISTRY_CLUSTER_PASS: &str = "alsofake";

pub static DEFAULTS_CLASS: &str = "easy";
pub static DEFAULTS_RESOURCES_CPU: i64 = 1;
pub static DEFAULTS_RESOURCES_MEMORY: &str = "500M";

pub static POINTS_EASY_CLASS: &str = "easy";
pub static POINTS_EASY_MIN: i64 = 200;
pub static POINTS_EASY_MAX: i64 = 500;
pub static POINTS_HARD_CLASS: &str = "hard";
pub static POINTS_HARD_MIN: i64 = 300;
pub static POINTS_HARD_MAX: i64 = 600;

pub static PROFILES_PROFILE_NAME: &str = "default";
pub static PROFILES_FRONTEND_URL: &str = "https://ctf.coolguy.invalid";
pub static PROFILES_FRONTEND_TOKEN: &str = "secretsecretsecret";
pub static PROFILES_CHALLENGES_DOMAIN: &str = "chals.coolguy.invalid";
pub static PROFILES_KUBECONTEXT: &str = "ctf-cluster";
pub static PROFILES_S3_BUCKET_NAME: &str = "ctf-bucket";
pub static PROFILES_S3_ENDPOINT: &str = "s3.coolguy.invalid";
pub static PROFILES_S3_REGION: &str = "us-west-2";
pub static PROFILES_S3_ACCESSKEY: &str = "accesskey";
pub static PROFILES_S3_SECRETACCESSKEY: &str = "secretkey";
