# Generated Kubernetes configuration (Linux/amd64)

This directory contains only deployment.yaml, service.yaml and this README.
The manifests use JSON syntax, a subset of YAML 1.2 accepted by Kubernetes.
Generation is local and deterministic on Linux and Windows. The Yetlet CLI
does not invoke kubectl, build, pull, publish or deploy anything, discover a
cluster, read kubeconfig or create a namespace. Windows generation still
targets Linux containers; Windows Kubernetes nodes are unsupported.

## Prepare the image separately

Use an unedited generated native-ssr-v1 cloud app with its explicit Cargo.lock.
Generate Docker configuration separately with `yetlet generate docker`. Follow
its README and inventory to prepare two physically separate, sanitized,
hash-verified named BuildKit contexts, app-src and yetlet-src. Never send the
live app or framework checkout as a build context. The configuration directory
is the only default context. An operator build has this explicit shape:

```sh
docker buildx build --platform linux/amd64 --load \
  --build-context app-src=APP_EXPORT \
  --build-context yetlet-src=FRAMEWORK_EXPORT \
  --file DOCKER_CONFIG/Dockerfile --tag EXPLICIT_IMAGE DOCKER_CONFIG
```

All uppercase placeholders require deliberate operator replacement. Build the
Linux/amd64 OCI image separately using the locked build, then explicitly import
it into each eligible node or push it to an operator-controlled registry. Supply
that exact accessible identity as --image when generating these manifests.
An imported tag is not proof that a digest reference resolves locally. The CLI
accepts only explicit registry/repository references with a non-latest tag or
sha256 digest and does not verify image contents, provenance or availability.
The imagePullPolicy is IfNotPresent: Kubernetes may pull a missing image when
the operator deploys. Registry authentication and image distribution belong to
the operator; no credentials or imagePullSecrets are generated.

## Inspect and choose a namespace explicitly

The package name is used unchanged for both resources and their selectors:
1-63 lowercase ASCII letters/digits/hyphens, beginning with a letter and ending
with a letter/digit. Underscores, truncation and translation are refused to avoid
collisions. Choose an existing namespace and review resource-name conflicts;
applying the same name in that namespace can update an existing workload.
Nothing here creates a namespace or embeds its name. Review your kubeconfig,
cluster context and namespace before choosing to run any of these commands.
From this output directory, replace EXPLICIT_CONTEXT and EXPLICIT_NAMESPACE:

```sh
kubectl --context EXPLICIT_CONTEXT --namespace EXPLICIT_NAMESPACE apply --dry-run=client -f deployment.yaml -f service.yaml
kubectl --context EXPLICIT_CONTEXT --namespace EXPLICIT_NAMESPACE apply --dry-run=server -f deployment.yaml -f service.yaml
kubectl --context EXPLICIT_CONTEXT --namespace EXPLICIT_NAMESPACE apply -f deployment.yaml -f service.yaml
```

These are optional operator actions, never CLI side effects. Even a client dry
run can fetch discovery/schema data; neither it nor a server dry run is an
offline guarantee or proof of a working Pod. Server dry run contacts the chosen
API server; the final command mutates resources in the selected namespace.

## Runtime limits and ownership

One replica is the default; --replicas accepts unsigned ASCII decimal 1-100.
Additional replicas do not establish high availability, spread or capacity.
The scheduler selects Linux/amd64 nodes. CPU request/limit 100m/500m and memory
request/limit 64Mi/128Mi are conservative starting examples, not measurements;
operators must size and test them for their workload.

The app container listens on 0.0.0.0:8080 using YETLET_HOST and YETLET_PORT. Its
named http port is selected by the internal ClusterIP Service, port 80 to http
(8080). There is no host port, external load balancer, TLS or Ingress assumption.
ClusterIP alone does not provide network isolation; cluster networking and
access policy belong to the operator.

Startup and liveness use /health/live; readiness uses /health/ready. These HTTP
probes reflect local process state only, not database, Redis or other dependency
health. No exec probe requires curl, wget or a shell in the image. Startup allows
about 60 seconds (30 attempts at 2 seconds); liveness runs every 10 seconds and
readiness every 5 seconds, each with a 1-second timeout and 3-failure threshold.

The accepted cloud app registers SIGTERM/SIGINT before bind and uses Axum
graceful shutdown. The 30-second terminationGracePeriodSeconds is an operator
budget, not a hard application drain guarantee: long requests/streams may exceed
it and be killed. Readiness/endpoint propagation is asynchronous. Measure stop
behavior and adjust the budget for real workloads; no preStop sleep or automatic
drain deadline is invented here.

The Pod runs as numeric UID/GID 10001, non-root, with RuntimeDefault seccomp and
no service-account token mount. The container has a read-only root filesystem,
no privilege escalation and drops all capabilities. The accepted Docker image
has an executable readable by this UID; the cloud app needs no writable files
or user entry. Operators using a different image must verify that compatibility.
No volumes, PVC, Secret, value-bearing ConfigMap, RBAC, HPA or extra resource is
generated. Secrets and application dependency readiness remain application and
operator responsibilities. This configuration is not production qualification.
