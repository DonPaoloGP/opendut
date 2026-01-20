use std::process::Command;
use std::str::FromStr;

use clap::ArgAction;
use cicero::path::repo_path;
use crate::core::types::Package;
use crate::core::util::RunRequiringSuccess;

/// A Docker tag
#[derive(Clone, Debug)]
pub struct DockerTag(pub String);

impl FromStr for DockerTag {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Build and publish a Docker image
#[derive(Debug, clap::Parser)]
pub struct DockerCli {
    /// Custom docker tag instead of the version of CARL
    #[arg(long)]
    pub tag: Option<DockerTag>,
    /// Publish container to docker registry
    #[arg(long, action = ArgAction::SetTrue)]
    pub publish: bool,
}

const OPENDUT_DOCKER_IMAGE_HOST: &str = "ghcr.io";
const OPENDUT_DOCKER_IMAGE_NAMESPACE: &str = "eclipse-opendut";

fn docker_container_uri(package: &Package, tag: &Option<DockerTag>) -> String {
    let image_host = std::env::var("OPENDUT_DOCKER_IMAGE_HOST").unwrap_or(OPENDUT_DOCKER_IMAGE_HOST.to_string());
    let image_namespace = std::env::var("OPENDUT_DOCKER_IMAGE_NAMESPACE").unwrap_or(OPENDUT_DOCKER_IMAGE_NAMESPACE.to_string());
    let version = match tag {
        None => { crate::build::PKG_VERSION }
        Some(tag) => {
            tag.0.as_str()
        }
    };
    let image_uri = format!("{}/{}/{}:{}", image_host, image_namespace, package.ident(), version);
    image_uri
}

pub fn build_docker_image(package: &Package, tag: Option<DockerTag>) -> crate::Result {
    let image_version_build_arg = format!("VERSION={}", crate::build::PKG_VERSION);
    let now = chrono::Utc::now().naive_utc();
    let container_uri = docker_container_uri(package, &tag);

    // https://github.com/opencontainers/image-spec/blob/main/annotations.md
    let source = format!("org.opencontainers.image.source={}", crate::core::metadata::repository_url());
    let url = format!("org.opencontainers.image.url={}", &container_uri);
    let version = format!("org.opencontainers.image.version={}", crate::build::PKG_VERSION);
    let created = format!("org.opencontainers.image.created={now}");
    let revision = format!("org.opencontainers.image.revision={}", crate::build::COMMIT_HASH);

    Command::new("docker")
        .current_dir(repo_path!())
        .args([
            "build",
            "--no-cache",
            "--file",
            &repo_path!(".ci/docker/carl/Dockerfile").display().to_string(),
            "--build-arg",
            &image_version_build_arg,
            "--label", &source,
            "--label", &url,
            "--label", &version,
            "--label", &created,
            "--label", &revision,
            "--tag",
            &container_uri,
            ".",
        ])
        .run_requiring_success()?;
    Ok(())
}


pub fn build_edgar_docker_image(tag: Option<DockerTag>) -> crate::Result {
    println!("hello World");
    Ok(())
}


pub fn publish_carl_docker_image(tag: Option<DockerTag>) -> crate::Result {
    let package = Package::Carl;
    Command::new("docker")
        .current_dir(repo_path!())
        .args(["push", &docker_container_uri(&package, &tag)])
        .run_requiring_success()?;
    Ok(())
}
