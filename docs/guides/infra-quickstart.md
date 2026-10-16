quick install instructions
requirements - cluster, registry, bucket
cluster-setup
configuring rcds.yaml


## Requirements

- Kubernetes cluster
  - Last tested with 1.37, any current version should work
  - The CNI must support `NetworkPolicy` resources (all the major clouds' managed clusters do)
  - Your cloud provider must support creating multiple load balancers.

  You must use a separate cluster for each deployment profile. The
  `cluster-setup` command will overwrite the DNS and loadbalancer configuration
  if multiple profiles are deployed into the same cluster.

- S3-compatible storage bucket
  - Private access only
  - Generate keypair credentials with read, write, presigned-link permissions

- OCI container registry
  - The registry *should* support auto-creating new repositories on push. Almost
    all do. 

    There are a couple known exceptions, notably AWS ECR. If you are using one
    of these that make you pre-create all container repository names ahead of
    time, you will need to use a custom [`registry.tag_format`
    template](../reference/rcds-yaml-reference.md#tagformat) and move the
    challenge name into the image tag.

  - BCDS expects there to be _a single_ registry shared between all profiles.
    Images are separated within the registry with separate tags for each
    profile.

- DNS subdomain zone for challenges delegated to your cloud provider
  - e.g. `chals.yourctf.example`
  - This must be unique across each deployment profile to avoid conflicts.







- BeaverCDS scoreboard site (https://github.com/osusec/beavercds-frontend)
  - 


## Create required infrastructure

1. Create a Kubernetes cluster in your cloud provider of choice
2. Create a S3 bucket
3. Create a OCI registry
4. Create a DNS zone for chals
