// Example strings for challenge.yaml

pub static CHAL_NAME: &str = "chalname";
pub static CHAL_AUTHOR: &str = "bagels";
pub static CHAL_DESC: &str = "this\nis a \nmultiline\ndescription";
pub static FILE_FLAG: &str = "src/flag";
pub static TEXT_FLAG: &str = "dam{xyz}";
pub static REGEX_FLAG: &str = "dam{.*}";
pub static PROVIDE_STR: &str = "handout/osint.jpg";
pub static PROVIDE_POD_FROM: &str = "podname";
pub static PROVIDE_POD_AS_FILE: &str = "files.zip";
pub static PROVIDE_POD_INCLUDE1: &str = "/lib/foo.so";
pub static PROVIDE_POD_INCLUDE2: &str = "/pwnme";
pub static PODS_NAME: &str = "podname";
pub static PODS_SRC_BUILD: &str = "src/";
pub static PODS_SRC_IMAGE: &str = "nginx:latest";
pub static PODS_ENV_KEY1: &str = "OPENAI_API_KEY";
pub static PODS_ENV_VAL1: &str = "sk-abc123jkjkjk";
pub static PODS_RESOURCES_CPU: &str = "2";
pub static PODS_RESOURCES_MEMORY: &str = "800M";
pub static PODS_REPLICAS: &str = "1";
pub static PODS_PORTS_INTERNAL_HTTP: &str = "8080";
pub static PODS_PORTS_INTERNAL_TCP: &str = "31337";
pub static PODS_PORTS_EXPOSE_HTTP: &str = "80";
pub static PODS_PORTS_EXPOSE_TCP: &str = "1337";
