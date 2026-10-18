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

### External-dns setup

You will need to provide the credentials and config for most of external-dns 
yourself, since these vary so much between clouds.

Bcds sets for you:
- sources:
- policy:
- domainFilters:
- registry: txt / txt*
- resources

You need to provide any cloud-specific config, e.g: 

for AWS:
```yaml
profiles:
  foo:
    #...
    dns:
      provider:
        name: aws
      aws:
        region: us-west-2
        zone-type: public
      # use IRSA to assume IAM role
      serviceAccount:
        annotations:
          eks.amazonaws.com/role-arn: arn:aws:iam::1234567890:role/irsa-external-dns-role
      # aws provider expects the filter to be the root of the hosted R53 zone
      domainFilters:
        - chals.myctf.domain
```

for DigitalOcean:
```yaml
profiles:
  bar:
    dns:

    TODO fill in from current ctf
```

### Validate config

here's an example app manifest to apply to check that your cluster is working
correctly: 

TODO, example chart? part of cluster-setup?


### Shared cluster with frontend

While this is possible, you will need to do extyra config to configure
external-dns if the frontend scoreboard domain is outside of the configured
`challenges_domain` (e.g. frontend at `ctf.my.domain`, chals at
`chals.ctf.my.domain`). External-dns is only be configured to watch for domains
under challenges_domain and will not pick up the frontend domain. You will
either need to: 
- update the helm release to update the watched domain outside of bcds
- install and configure a separate external-dns instance (and expose the frontend on
  a non-overlapping domain)
- manage that record some other way

The managed ingress controller and cert-manager do not have any problems and can
be used to serve non-challenge applications as-is (with the corresponding
ingress class set).
